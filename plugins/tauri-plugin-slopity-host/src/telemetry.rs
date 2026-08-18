use crate::HostDeviceTelemetry;

const KIB_PER_MIB: u64 = 1_024;

#[cfg(target_os = "linux")]
pub(crate) fn host_device_telemetry() -> HostDeviceTelemetry {
    match std::fs::read_to_string("/proc/meminfo") {
        Ok(meminfo) => {
            let (total_memory_mib, available_memory_mib) = parse_linux_meminfo(&meminfo);
            HostDeviceTelemetry {
                platform: "linux".into(),
                source: "linux:/proc/meminfo".into(),
                total_memory_mib,
                available_memory_mib,
                ..HostDeviceTelemetry::default()
            }
        }
        Err(_) => HostDeviceTelemetry::unavailable("linux", "linux:/proc/meminfo-unavailable"),
    }
}

#[cfg(all(not(target_os = "android"), not(target_os = "linux")))]
pub(crate) fn host_device_telemetry() -> HostDeviceTelemetry {
    HostDeviceTelemetry::unavailable(std::env::consts::OS, "platform-memory-provider-unavailable")
}

fn parse_linux_meminfo(meminfo: &str) -> (Option<u64>, Option<u64>) {
    let mut total_memory_mib = None;
    let mut available_memory_mib = None;

    for line in meminfo.lines() {
        if total_memory_mib.is_none() {
            total_memory_mib = parse_kib_field(line, "MemTotal:");
        }
        if available_memory_mib.is_none() {
            available_memory_mib = parse_kib_field(line, "MemAvailable:");
        }
        if total_memory_mib.is_some() && available_memory_mib.is_some() {
            break;
        }
    }

    (total_memory_mib, available_memory_mib)
}

fn parse_kib_field(line: &str, field: &str) -> Option<u64> {
    let remainder = line.strip_prefix(field)?.trim();
    let mut parts = remainder.split_whitespace();
    let kib = parts.next()?.parse::<u64>().ok()?;
    if parts.next()? != "kB" {
        return None;
    }
    Some(kib / KIB_PER_MIB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_linux_like_memory_snapshot() {
        let meminfo = "MemTotal:        16384000 kB\nMemFree:          100000 kB\nMemAvailable:     8192000 kB\nCached:            500000 kB\n";
        let (total, available) = parse_linux_meminfo(meminfo);

        assert_eq!(total, Some(16_000));
        assert_eq!(available, Some(8_000));
    }

    #[test]
    fn incomplete_or_unexpected_meminfo_is_reported_as_unavailable() {
        let (total, available) =
            parse_linux_meminfo("MemTotal: 2048000 bytes\nMemAvailable: not-a-number kB\n");

        assert_eq!(total, None);
        assert_eq!(available, None);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod live_linux_tests {
    use super::*;

    #[test]
    fn live_linux_telemetry_reports_structurally_valid_memory() {
        let telemetry = host_device_telemetry();
        let total = telemetry
            .total_memory_mib
            .expect("Linux /proc/meminfo should report MemTotal");
        let available = telemetry
            .available_memory_mib
            .expect("Linux /proc/meminfo should report MemAvailable");

        assert_eq!(telemetry.platform, "linux");
        assert!(total > 0);
        assert!(available <= total);
    }
}
