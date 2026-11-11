#!/usr/bin/env bash
# Verify generated Android Gradle JVM tests and fail closed when lifecycle tests exist.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
generated_android="$repo_root/apps/slopity/src-tauri/gen/android"
host_android="$repo_root/plugins/tauri-plugin-slopity-host/android"
expected_test="$host_android/src/test/java/com/slopity/host/HostForegroundServiceTest.kt"

if [[ ! -f "$generated_android/gradlew" ]]; then
  echo "::error::Generated Android Gradle wrapper missing; run Tauri Android build before JVM unit tests." >&2
  exit 1
fi

# Invoke Gradle's multi-project task after the Tauri Android build has generated
# plugin module wiring (including tauri.settings.gradle). A 'NO-SOURCE' result
# must never be counted as lifecycle coverage when the tests exist.
echo "::group::Generated Android Gradle debug JVM tests"
(
  cd "$generated_android"
  bash ./gradlew --no-daemon --console=plain testDebugUnitTest
)
echo "::endgroup::"

if [[ ! -f "$expected_test" ]]; then
  echo "::warning::HostForegroundServiceTest.kt is absent on this branch. The generated Gradle task ran, but Android host lifecycle JUnit coverage is NOT certified. Integrate Android PR #51 and rerun CI."
  exit 0
fi

# Verify execution of the *specific* plugin test class, not merely another
# module's tests, a zero-test Gradle success, or a successful Kotlin compile.
python3 - "$generated_android" "$host_android" <<'PY'
import pathlib
import sys
import xml.etree.ElementTree as ET

generated, plugin = map(pathlib.Path, sys.argv[1:])
report_files = sorted({
    path for root in (generated, plugin)
    for path in root.rglob("TEST-*.xml")
    if path.parent.name == "testDebugUnitTest"
    and path.parent.parent.name == "test-results"
})
if not report_files:
    sys.exit("No testDebugUnitTest XML reports found; refusing to certify Kotlin JUnit tests")

expected = "com.slopity.host.HostForegroundServiceTest"
executed = 0
failures = 0
errors = 0
skipped = 0
for report in report_files:
    root = ET.parse(report).getroot()
    suites = [root] if root.tag == "testsuite" else list(root.findall(".//testsuite"))
    for suite in suites:
        suite_name = suite.attrib.get("name", "")
        cases = suite.findall("testcase")
        matching_cases = [
            case for case in cases
            if case.attrib.get("classname") == expected
        ]
        if suite_name != expected and not matching_cases:
            continue
        # The suite reported by Gradle is authoritative for class-level counts.
        executed += int(suite.attrib.get("tests", len(cases)))
        failures += int(suite.attrib.get("failures", 0))
        errors += int(suite.attrib.get("errors", 0))
        skipped += int(suite.attrib.get("skipped", 0))

if executed < 1 or failures or errors or skipped:
    sys.exit(
        f"Android host JUnit gate failed: tests={executed} "
        f"failures={failures} errors={errors} skipped={skipped}"
    )
print(f"Verified Android host lifecycle JUnit suite: {executed} passed, zero failures/errors/skips")
PY
