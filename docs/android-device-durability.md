# Android physical-device durability validation

This procedure turns Slopity's Android durability claim into something reproducible and reviewable. Compilation, an emulator, a merged manifest, and an installable APK are useful gates, but none of them prove that an OEM will keep a user-started local server reachable for hours with the screen off.

Do not mark Android durable hosting as proven until raw evidence from at least one physical ARM64 device is attached to a test record. Do not invent values when a device or Android API does not expose them; record `unavailable` instead.

## What this validates

The test covers the Android/Tauri/native host boundary for the built-in Rust HTTP probe only:

- debug APK installation and launch;
- loopback-first hosting and optional LAN hosting;
- foreground notification visibility and active-server count;
- UI backgrounding and screen-off reachability;
- native stop-request reconciliation;
- return-to-UI state reconciliation;
- battery, battery temperature, thermal status, memory, and storage observations;
- clean stop and port release;
- multi-hour survival on a real device.

It does **not** validate Minecraft, JVM/Java downloads, Jellyfin, Node.js, Python, PHP, arbitrary processes, production signing, Google Play policy approval, process-death recovery, reboot recovery, or vendor-specific background bypasses.

## Prerequisites

On the host computer:

- Android Platform Tools (`adb`);
- `curl`;
- a USB cable or trusted wireless ADB connection;
- the ARM64 debug APK produced by Slopity CI;
- this repository checkout, so `scripts/android-durability-probe.sh` is available.

On the device:

- ARM64 Android 8.0 / API 26 or newer;
- developer options and USB debugging enabled for the test session;
- enough free storage to install the debug APK;
- no unrelated benchmark, game, charger-temperature test, or heavy workload running during the baseline.

Record whether the device is charging. Do not compare battery drain between runs that use materially different charging or screen conditions.

## 1. Record device identity before installation

Run:

```bash
bash scripts/android-durability-probe.sh info | tee device-info.txt
```

Keep the raw output. At minimum the final evidence record must include manufacturer, model, Android version, API level, ABI, build fingerprint, and whether this is stock/AOSP-ish Android, Samsung software, Xiaomi/Poco software, or another OEM image.

## 2. Install the exact CI APK

Download the APK from the GitHub Actions artifact associated with the tested commit. Record the workflow run, commit SHA, artifact name, and APK SHA-256.

Install it with:

```bash
bash scripts/android-durability-probe.sh install path/to/app-universal-debug.apk
```

If the device already has a debug Slopity build, `adb install -r` replaces it while preserving app data. For a clean-install test, explicitly uninstall Slopity first and record that choice.

## 3. Create a loopback BuiltInHttp profile

In Slopity:

1. Create or enable a `BuiltInHttp` profile.
2. Keep network scope on loopback.
3. Choose and record the port, for example `8080`.
4. Start the server through the normal UI.
5. Grant notification permission when Android prompts on Android 13+.

Expected behavior:

- start is user initiated;
- the server reports running;
- the foreground notification appears and remains ongoing;
- the notification says one server is active;
- denying notification permission, disabling app notifications, or disabling the hosting notification channel must not be treated as equivalent to safe visible hosting.

Capture a screenshot of the notification and a Slopity dashboard screenshot. Screenshots are supporting evidence, not a substitute for reachability logs.

## 4. Verify loopback reachability through ADB forwarding

A loopback listener on the phone is not directly reachable at the host computer's `127.0.0.1`. Forward a host port to the device port:

```bash
adb forward tcp:8080 tcp:8080
curl --fail --show-error http://127.0.0.1:8080/health
```

Use the actual profile port if it is not `8080`.

Start the evidence sampler:

```bash
bash scripts/android-durability-probe.sh probe \
  http://127.0.0.1:8080/health \
  14400 \
  60 \
  evidence-loopback
```

Arguments are `URL`, total duration in seconds, interval in seconds, and output directory. `14400` is four hours. For the minimum one-hour roadmap proof use at least `3600`; a multi-hour target is strongly preferred for durability work.

The sampler records reachability plus raw `adb shell dumpsys battery`, `dumpsys thermalservice`, selected `/proc/meminfo`, and `/data` storage observations. Keep the raw directory unchanged when attaching evidence.

## 5. Background the UI

While the probe is running:

1. Press Home so Slopity is no longer foreground UI.
2. Confirm the Slopity hosting notification remains visible.
3. Leave the server running for at least several probe intervals.
4. Confirm `reachability.csv` continues to report HTTP success.

Record any missed probes with timestamps rather than deleting them.

## 6. Turn the screen off

Turn the display off normally and keep the device untouched for a meaningful interval. For a multi-hour run, the majority of the run should include realistic background/screen-off time rather than keeping Slopity open on screen.

Expected evidence:

- health probes continue to succeed;
- the process is not repeatedly restarted;
- battery and temperature remain observable when the device exposes them;
- thermal events are recorded rather than hidden;
- no claim is made from a single short successful interval.

Do not use `adb shell svc power stayon`, wakelock hacks, root-only keepalive tools, hidden services, or OEM-specific bypasses. They would invalidate the product behavior being tested.

## 7. Optional LAN profile

After the loopback test, optionally create a separate `BuiltInHttp` profile using LAN scope. Record the phone's Wi-Fi IPv4 address and the port, then probe from another device on the same trusted LAN:

```bash
curl --fail --show-error http://PHONE_LAN_IP:PORT/health
```

For a long LAN run, the probe host should remain on the same LAN. Do not expose the debug server to the public internet, guest networks, or untrusted Wi-Fi.

Record whether Android changed networks, Wi-Fi slept, DHCP changed the address, or the access point isolated clients. Those are distinct from Slopity process failures.

## 8. Exercise multiple-server notification reconciliation

When device resources permit, start a second harmless `BuiltInHttp` profile on a different port.

Verify:

1. both servers are reachable;
2. the notification changes from one server to two servers;
3. stopping one profile leaves the other reachable;
4. the notification returns to one server;
5. stopping the final server removes the hosting notification.

A stale count or a stale notification after the final server stops is a failure. Record it with timestamp and reproduction steps.

## 9. Return to Slopity and verify state reconciliation

After background and screen-off time:

1. unlock the phone;
2. open Slopity from the launcher or notification;
3. request/refresh the dashboard state;
4. confirm the displayed running/stopped states match actual reachability;
5. confirm the native host-service count matches the authoritative Rust server count.

If Android killed the foreground service while a Rust listener remained active, a fresh observation should attempt to restore foreground state. If foreground state cannot be restored, Slopity is expected to fail conservatively rather than intentionally continue invisible hosting.

If the entire app process was killed, record that separately. Current Slopity hosting is in-process and does not claim process-death recovery.

## 10. Stop through the normal UI

Stop the server through Slopity.

Verify:

- the server transitions to stopped;
- the final-server stop removes the foreground notification;
- the health endpoint is no longer reachable;
- the port is released.

Useful checks:

```bash
curl --fail --show-error --max-time 5 http://127.0.0.1:8080/health
adb shell 'command -v ss >/dev/null && ss -ltn || true'
```

The `curl` command is expected to fail after a clean stop. For a loopback test, keep the same `adb forward` in place so failure means the device listener is gone rather than the forwarding rule disappearing.

## 11. Repeat using the notification stop action

Start the server again and verify it is reachable. Tap **Stop safely** in the persistent notification.

Expected behavior:

1. the native service records a stop request;
2. Slopity opens for safe Rust-side reconciliation;
3. the application stops active hosted listeners before removing the foreground service;
4. the port is released;
5. the notification disappears after the final listener stops.

The notification action intentionally does not kill the Kotlin service first while Rust listeners are still active. If the app fails to open, the request remains pending, the server remains foregrounded, and that result must be recorded as a failure/limitation rather than forcing a hidden stop.

## 12. Observe battery and thermal behavior

Use both Slopity telemetry and the raw ADB evidence. Record:

- battery percentage at start and end;
- charging state;
- maximum observed battery temperature when available;
- Android thermal status transitions when available;
- any severe/critical/emergency/shutdown thermal state;
- any obvious throttling or reachability loss correlated with thermal state.

Do not infer CPU temperature from battery temperature. They are different measurements. If an OEM does not expose a field, write `unavailable`.

## 13. OEM validation expectations

### Stock/AOSP-ish Android

Use this as the standards-baseline category when possible. Do not whitelist Slopity from battery optimization unless the product explicitly requires and documents that as a user setup step in a future change. Record default settings used during the test.

### Samsung

Test with normal default device-care/battery settings first. Record One UI version and any battery mode shown by the device. If Samsung stops the service, preserve the evidence and classify it before considering product changes. Do not add Samsung-specific hidden keepalive behavior from one anecdotal failure.

### Xiaomi / Poco

Record MIUI or HyperOS version and the app's battery-management setting. Test default behavior first. Xiaomi/Poco devices are useful for exposing aggressive OEM process management, but a failure is evidence to investigate, not permission to bypass Android's foreground-service model or silently request broad privileges.

For every OEM, a second run with a user-selected battery exception may be useful diagnostically, but it must be labeled separately from the default-settings result.

## Evidence record template

Create one record per device/build/test condition. YAML is suggested because it is easy to review, but JSON or a Markdown table is acceptable if the same fields are retained.

```yaml
test:
  date_utc:
  tester:
  slopity_commit:
  github_actions_run:
  artifact_name:
  apk_sha256:
  test_duration:
  interval_seconds:
  network_scope: loopback # or lan
  server_ports: []

device:
  manufacturer:
  model:
  android_version:
  api_level:
  abi:
  build_fingerprint:
  software_family: # stock-aosp-ish, samsung-one-ui, xiaomi-hyperos, poco-hyperos, other
  battery_management_setting:

observations:
  battery_start:
  battery_end:
  charging:
  max_temperature_celsius:
  thermal_events: []
  background_reachable:
  screen_off_reachable:
  notification_persistent:
  notification_server_count_correct:
  ui_reconciliation:
  stop_action:
  ui_stop_released_port:
  notification_stop_released_port:
  process_killed:
  service_killed:
  unavailable_metrics: []
  failures: []

artifacts:
  raw_probe_directory:
  reachability_csv:
  telemetry_log:
  notification_screenshot:
  dashboard_screenshot:
  notes:
```

Do not replace failed or missing fields with zero. Use `unavailable`, `not_observed`, or a concrete failure description.

## Pass/fail guidance

A strong physical-device durability result requires all of the following for the tested device/build/conditions:

- installation succeeds;
- user-started server is reachable;
- foreground notification is persistently visible while hosting;
- reachability survives UI backgrounding;
- reachability survives substantial screen-off time;
- active-server counts reconcile correctly;
- return-to-UI state matches actual runtime state;
- UI stop releases the port;
- notification stop request safely reconciles and releases the port;
- no unexplained process death or long reachability outage occurs during the target duration;
- raw battery/thermal evidence is retained, with unavailable values explicitly marked.

Passing one device does not prove every OEM. It proves only the exact tested device/software/settings/build combination and provides the first evidence needed to widen the support claim responsibly.
