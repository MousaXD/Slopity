package com.pockethost.app.capability

import android.app.ActivityManager
import android.content.Context
import android.os.Build
import android.os.PowerManager
import android.os.StatFs
import com.pockethost.app.domain.DeviceCapabilities
import com.pockethost.app.domain.ThermalStatus

class DeviceCapabilityProbe(private val context: Context) {
    fun read(): DeviceCapabilities {
        val activityManager = context.getSystemService(ActivityManager::class.java)
        val memoryInfo = ActivityManager.MemoryInfo().also(activityManager::getMemoryInfo)
        val statFs = StatFs(context.filesDir.absolutePath)
        val powerManager = context.getSystemService(PowerManager::class.java)

        return DeviceCapabilities(
            totalMemoryMb = memoryInfo.totalMem.toMbInt(),
            availableMemoryMb = memoryInfo.availMem.toMbInt(),
            cpuCores = Runtime.getRuntime().availableProcessors().coerceAtLeast(1),
            supportedAbis = Build.SUPPORTED_ABIS.toList(),
            freeStorageMb = statFs.availableBytes / BYTES_PER_MB,
            thermalStatus = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                powerManager.currentThermalStatus.toDomainThermalStatus()
            } else {
                ThermalStatus.UNKNOWN
            },
        )
    }

    private fun Long.toMbInt(): Int =
        (this / BYTES_PER_MB).coerceAtMost(Int.MAX_VALUE.toLong()).toInt()

    private fun Int.toDomainThermalStatus(): ThermalStatus = when (this) {
        PowerManager.THERMAL_STATUS_NONE -> ThermalStatus.NONE
        PowerManager.THERMAL_STATUS_LIGHT -> ThermalStatus.LIGHT
        PowerManager.THERMAL_STATUS_MODERATE -> ThermalStatus.MODERATE
        PowerManager.THERMAL_STATUS_SEVERE -> ThermalStatus.SEVERE
        PowerManager.THERMAL_STATUS_CRITICAL -> ThermalStatus.CRITICAL
        PowerManager.THERMAL_STATUS_EMERGENCY -> ThermalStatus.EMERGENCY
        PowerManager.THERMAL_STATUS_SHUTDOWN -> ThermalStatus.SHUTDOWN
        else -> ThermalStatus.UNKNOWN
    }

    private companion object {
        const val BYTES_PER_MB = 1_048_576L
    }
}
