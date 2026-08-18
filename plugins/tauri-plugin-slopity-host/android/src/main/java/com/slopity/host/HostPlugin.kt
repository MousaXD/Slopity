package com.slopity.host

import android.Manifest
import android.app.Activity
import android.app.ActivityManager
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.os.BatteryManager
import android.os.Build
import android.os.PowerManager
import android.os.StatFs
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class StartHostingArgs {
    var label: String? = null
    var activeServerCount: Int? = null
}

@TauriPlugin(
    permissions = [
        Permission(
            strings = [Manifest.permission.POST_NOTIFICATIONS],
            alias = "postNotification"
        )
    ]
)
class HostPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun deviceTelemetry(invoke: Invoke) {
        val appContext = activity.applicationContext
        val result = JSObject().apply {
            put("platform", "android")
            put("source", "android-system-services")
        }

        val activityManager = appContext.getSystemService(Context.ACTIVITY_SERVICE) as? ActivityManager
        activityManager?.let { manager ->
            val memory = ActivityManager.MemoryInfo()
            manager.getMemoryInfo(memory)
            result.put("totalMemoryMib", memory.totalMem / BYTES_PER_MIB)
            result.put("availableMemoryMib", memory.availMem / BYTES_PER_MIB)
        }

        val batteryIntent = appContext.registerReceiver(
            null,
            IntentFilter(Intent.ACTION_BATTERY_CHANGED)
        )
        batteryIntent?.let { battery ->
            val level = battery.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
            val scale = battery.getIntExtra(BatteryManager.EXTRA_SCALE, -1)
            if (level >= 0 && scale > 0) {
                result.put(
                    "batteryPercentage",
                    ((level * 100L) / scale).coerceIn(0L, 100L).toInt()
                )
            }

            val status = battery.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
            if (status != -1) {
                result.put(
                    "charging",
                    status == BatteryManager.BATTERY_STATUS_CHARGING ||
                        status == BatteryManager.BATTERY_STATUS_FULL
                )
            }

            val temperatureTenthsCelsius =
                battery.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, Int.MIN_VALUE)
            if (temperatureTenthsCelsius != Int.MIN_VALUE) {
                result.put("batteryTemperatureCelsius", temperatureTenthsCelsius / 10.0)
            }
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            val powerManager = appContext.getSystemService(Context.POWER_SERVICE) as? PowerManager
            powerManager?.let { manager ->
                result.put("thermalStatus", thermalStatusLabel(manager.currentThermalStatus))
            }
        }

        runCatching {
            StatFs(appContext.filesDir.absolutePath).availableBytes / BYTES_PER_MIB
        }.getOrNull()?.let { freeStorageMib ->
            result.put("freeStorageMib", freeStorageMib)
        }

        invoke.resolve(result)
    }

    @Command
    fun startHosting(invoke: Invoke) {
        val args = invoke.parseArgs(StartHostingArgs::class.java)
        val label = args.label?.takeIf { it.isNotBlank() } ?: "Hosting a Slopity server"
        val activeServerCount = (args.activeServerCount ?: 1).coerceAtLeast(1)
        HostForegroundService.markStartRequested(label, activeServerCount)

        val intent = Intent(activity.applicationContext, HostForegroundService::class.java)
            .setAction(HostForegroundService.ACTION_START_OR_UPDATE)
            .putExtra(HostForegroundService.EXTRA_LABEL, label)
            .putExtra(HostForegroundService.EXTRA_ACTIVE_SERVER_COUNT, activeServerCount)
        ContextCompat.startForegroundService(activity.applicationContext, intent)
        invoke.resolve(status())
    }

    @Command
    fun stopHosting(invoke: Invoke) {
        val stopped = activity.applicationContext.stopService(
            Intent(activity.applicationContext, HostForegroundService::class.java)
        )
        HostForegroundService.markStopped()
        val reason = if (stopped) {
            "Android foreground host service stopped after the application stopped its hosted servers."
        } else {
            "Android foreground host service was already stopped."
        }
        invoke.resolve(status(reason))
    }

    @Command
    fun getStatus(invoke: Invoke) {
        invoke.resolve(status())
    }

    private fun thermalStatusLabel(status: Int): String {
        return when (status) {
            PowerManager.THERMAL_STATUS_NONE -> "none"
            PowerManager.THERMAL_STATUS_LIGHT -> "light"
            PowerManager.THERMAL_STATUS_MODERATE -> "moderate"
            PowerManager.THERMAL_STATUS_SEVERE -> "severe"
            PowerManager.THERMAL_STATUS_CRITICAL -> "critical"
            PowerManager.THERMAL_STATUS_EMERGENCY -> "emergency"
            PowerManager.THERMAL_STATUS_SHUTDOWN -> "shutdown"
            else -> "unknown-$status"
        }
    }

    private fun status(reasonOverride: String? = null): JSObject {
        val snapshot = HostForegroundService.snapshot()
        val notificationPermissionRequired = Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU
        val notificationPermissionGranted = !notificationPermissionRequired ||
            ContextCompat.checkSelfPermission(
                activity.applicationContext,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
        val notificationManager = activity.applicationContext
            .getSystemService(NotificationManager::class.java)
        val appNotificationsEnabled = notificationManager.areNotificationsEnabled()
        val channel = notificationManager.getNotificationChannel(HostForegroundService.CHANNEL_ID)
        val channelEnabled = channel == null || channel.importance != NotificationManager.IMPORTANCE_NONE
        val notificationVisible = snapshot.active &&
            notificationPermissionGranted &&
            appNotificationsEnabled &&
            channelEnabled

        val reason = reasonOverride ?: when {
            snapshot.stopRequestPending ->
                "Hosting is still active. A notification stop request is pending; Slopity must reconcile and stop its Rust servers before the foreground service can safely stop."
            snapshot.startRequestPending && !snapshot.active ->
                "Android foreground hosting was requested and is waiting for the service to enter the foreground."
            !snapshot.active ->
                "Android foreground host service is not active."
            notificationPermissionRequired && !notificationPermissionGranted ->
                "Hosting is active, but POST_NOTIFICATIONS is denied, so the foreground notification is not visible in the notification drawer."
            !appNotificationsEnabled ->
                "Hosting is active, but application notifications are disabled in Android settings."
            !channelEnabled ->
                "Hosting is active, but the Slopity hosting notification channel is disabled."
            notificationVisible ->
                "Android foreground hosting is active and its persistent notification should be visible."
            else ->
                "Android foreground hosting is active, but notification visibility could not be confirmed."
        }

        return JSObject().apply {
            put("platform", "android")
            put("active", snapshot.active)
            put("notificationVisible", notificationVisible)
            put("notificationPermissionGranted", notificationPermissionGranted)
            put("notificationPermissionRequired", notificationPermissionRequired)
            put("label", snapshot.label)
            put("activeServerCount", snapshot.activeServerCount)
            put("stopRequestPending", snapshot.stopRequestPending)
            put("reason", reason)
        }
    }

    companion object {
        private const val BYTES_PER_MIB = 1024L * 1024L
    }
}
