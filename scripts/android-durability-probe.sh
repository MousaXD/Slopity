#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/android-durability-probe.sh info
  bash scripts/android-durability-probe.sh install <apk>
  bash scripts/android-durability-probe.sh probe <url> [duration_seconds] [interval_seconds] [output_dir]
  bash scripts/android-durability-probe.sh ports

Environment:
  ANDROID_SERIAL   Optional adb device serial when more than one device is connected.

Examples:
  bash scripts/android-durability-probe.sh info
  bash scripts/android-durability-probe.sh install app-universal-debug.apk
  adb forward tcp:8080 tcp:8080
  bash scripts/android-durability-probe.sh probe http://127.0.0.1:8080/health 14400 60 evidence-loopback
EOF
}

fail() {
  echo "error: $*" >&2
  exit 1
}

command -v adb >/dev/null 2>&1 || fail "adb is required"

ADB=(adb)
if [[ -n "${ANDROID_SERIAL:-}" ]]; then
  ADB+=( -s "$ANDROID_SERIAL" )
fi

adb_shell() {
  "${ADB[@]}" shell "$@"
}

require_device() {
  "${ADB[@]}" get-state >/dev/null 2>&1 || fail "no authorized Android device is available"
}

prop() {
  local key="$1"
  adb_shell getprop "$key" 2>/dev/null | tr -d '\r'
}

print_info() {
  require_device
  cat <<EOF
captured_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)
serial: $("${ADB[@]}" get-serialno 2>/dev/null || echo unavailable)
manufacturer: $(prop ro.product.manufacturer)
model: $(prop ro.product.model)
android_version: $(prop ro.build.version.release)
api_level: $(prop ro.build.version.sdk)
abi: $(prop ro.product.cpu.abi)
abi_list: $(prop ro.product.cpu.abilist)
build_fingerprint: $(prop ro.build.fingerprint)
security_patch: $(prop ro.build.version.security_patch)
EOF

  echo
  echo "--- battery ---"
  adb_shell dumpsys battery 2>/dev/null || true
  echo
  echo "--- thermalservice ---"
  adb_shell dumpsys thermalservice 2>/dev/null || echo "unavailable"
  echo
  echo "--- memory ---"
  adb_shell sh -c "grep -E '^(MemTotal|MemAvailable):' /proc/meminfo" 2>/dev/null || echo "unavailable"
  echo
  echo "--- storage /data ---"
  adb_shell df -k /data 2>/dev/null || echo "unavailable"
}

install_apk() {
  require_device
  local apk="${1:-}"
  [[ -n "$apk" ]] || fail "install requires an APK path"
  [[ -f "$apk" ]] || fail "APK not found: $apk"
  echo "APK SHA-256:"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$apk"
  else
    echo "sha256sum unavailable on host"
  fi
  "${ADB[@]}" install -r "$apk"
}

print_ports() {
  require_device
  if adb_shell sh -c "command -v ss >/dev/null 2>&1"; then
    adb_shell ss -ltn
  elif adb_shell sh -c "command -v netstat >/dev/null 2>&1"; then
    adb_shell netstat -ltn
  else
    echo "Neither ss nor netstat is available on this Android image. Record port release with reachability failure instead."
  fi
}

validate_positive_integer() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+$ ]] || fail "$name must be an integer"
  (( value > 0 )) || fail "$name must be greater than zero"
}

probe() {
  require_device
  command -v curl >/dev/null 2>&1 || fail "curl is required for reachability probing"

  local url="${1:-}"
  local duration="${2:-14400}"
  local interval="${3:-60}"
  local output_dir="${4:-android-durability-evidence-$(date -u +%Y%m%dT%H%M%SZ)}"

  [[ -n "$url" ]] || fail "probe requires a URL"
  validate_positive_integer duration_seconds "$duration"
  validate_positive_integer interval_seconds "$interval"

  mkdir -p "$output_dir"
  local reachability="$output_dir/reachability.csv"
  local telemetry="$output_dir/telemetry.log"
  local metadata="$output_dir/probe-metadata.txt"

  {
    echo "started_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "url: $url"
    echo "duration_seconds: $duration"
    echo "interval_seconds: $interval"
    echo "android_serial: $("${ADB[@]}" get-serialno 2>/dev/null || echo unavailable)"
    echo "manufacturer: $(prop ro.product.manufacturer)"
    echo "model: $(prop ro.product.model)"
    echo "android_version: $(prop ro.build.version.release)"
    echo "api_level: $(prop ro.build.version.sdk)"
    echo "abi: $(prop ro.product.cpu.abi)"
    echo "build_fingerprint: $(prop ro.build.fingerprint)"
  } > "$metadata"

  echo "timestamp_utc,reachable,http_code,total_seconds" > "$reachability"
  local start_epoch
  start_epoch=$(date +%s)
  local deadline=$((start_epoch + duration))

  while (( $(date +%s) < deadline )); do
    local now_iso now_epoch elapsed reachable http_code
    now_iso=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    now_epoch=$(date +%s)
    elapsed=$((now_epoch - start_epoch))
    reachable=0
    http_code="000"

    if http_code=$(curl --silent --show-error --output /dev/null \
        --write-out '%{http_code}' --max-time 10 "$url"); then
      if [[ "$http_code" =~ ^2[0-9][0-9]$ ]]; then
        reachable=1
      fi
    else
      http_code="curl-error"
    fi
    echo "$now_iso,$reachable,$http_code,$elapsed" >> "$reachability"

    {
      echo "=== sample $now_iso elapsed_seconds=$elapsed ==="
      echo "--- battery ---"
      adb_shell dumpsys battery 2>/dev/null || echo "unavailable"
      echo "--- thermalservice ---"
      adb_shell dumpsys thermalservice 2>/dev/null || echo "unavailable"
      echo "--- memory ---"
      adb_shell sh -c "grep -E '^(MemTotal|MemAvailable):' /proc/meminfo" 2>/dev/null || echo "unavailable"
      echo "--- storage /data ---"
      adb_shell df -k /data 2>/dev/null || echo "unavailable"
      echo
    } >> "$telemetry"

    local after_sample
    after_sample=$(date +%s)
    if (( after_sample >= deadline )); then
      break
    fi
    local remaining=$((deadline - after_sample))
    if (( remaining < interval )); then
      sleep "$remaining"
    else
      sleep "$interval"
    fi
  done

  {
    echo "finished_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "reachability_csv: $reachability"
    echo "telemetry_log: $telemetry"
  } >> "$metadata"

  echo "Evidence written to: $output_dir"
  echo "Do not edit failed samples out of $reachability. Attach the raw directory to the device test record."
}

case "${1:-}" in
  info)
    print_info
    ;;
  install)
    shift
    install_apk "${1:-}"
    ;;
  probe)
    shift
    probe "$@"
    ;;
  ports)
    print_ports
    ;;
  -h|--help|help|"")
    usage
    ;;
  *)
    usage >&2
    fail "unknown command: $1"
    ;;
esac
