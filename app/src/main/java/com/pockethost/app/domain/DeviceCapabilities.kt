package com.pockethost.app.domain

data class DeviceCapabilities(
    val totalMemoryMb: Int,
    val availableMemoryMb: Int,
    val cpuCores: Int,
    val supportedAbis: List<String>,
    val freeStorageMb: Long,
    val thermalStatus: ThermalStatus,
)

enum class ThermalStatus {
    UNKNOWN,
    NONE,
    LIGHT,
    MODERATE,
    SEVERE,
    CRITICAL,
    EMERGENCY,
    SHUTDOWN,
}

enum class DeviceTier {
    LIMITED,
    STANDARD,
    STRONG,
    EXTREME,
}

data class HostingPlan(
    val tier: DeviceTier,
    val reservedForAndroidMb: Int,
    val usableForServersMb: Int,
    val recommendedMaxConcurrentServers: Int,
    val recommendedMemoryPerServerMb: Int,
    val warnings: List<String>,
)

object CapabilityAdvisor {
    fun advise(capabilities: DeviceCapabilities): HostingPlan {
        val total = capabilities.totalMemoryMb.coerceAtLeast(0)
        val available = capabilities.availableMemoryMb.coerceIn(0, total.coerceAtLeast(0))
        val reserve = maxOf(1_536, total / 4).coerceAtMost(total)
        val usableByTotal = (total - reserve).coerceAtLeast(0)
        val usable = minOf(available, usableByTotal)
        val memorySlots = usable / 768
        val cpuSlots = (capabilities.cpuCores - 1).coerceAtLeast(0)
        val maxConcurrent = minOf(memorySlots, cpuSlots, 6).coerceAtLeast(0)
        val perServer = if (maxConcurrent == 0) 0 else usable / maxConcurrent
        val tier = when {
            total < 4_096 || capabilities.cpuCores < 4 -> DeviceTier.LIMITED
            total < 8_192 || capabilities.cpuCores < 6 -> DeviceTier.STANDARD
            total < 12_288 || capabilities.cpuCores < 8 -> DeviceTier.STRONG
            else -> DeviceTier.EXTREME
        }
        val warnings = buildList {
            if (maxConcurrent == 0) add("Current free resources are too low for a recommended server session.")
            if (capabilities.freeStorageMb < 4_096) add("Less than 4 GB of free app storage is available.")
            if (capabilities.thermalStatus >= ThermalStatus.SEVERE) {
                add("The device is already thermally constrained; do not start a workload.")
            }
            if (capabilities.supportedAbis.none { it == "arm64-v8a" }) {
                add("ARM64 is not reported; most planned runtime packages will be unavailable.")
            }
        }

        return HostingPlan(
            tier = tier,
            reservedForAndroidMb = reserve,
            usableForServersMb = usable,
            recommendedMaxConcurrentServers = maxConcurrent,
            recommendedMemoryPerServerMb = perServer,
            warnings = warnings,
        )
    }
}
