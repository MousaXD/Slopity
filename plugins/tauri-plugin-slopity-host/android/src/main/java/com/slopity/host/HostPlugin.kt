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
    init {
        // New Activity creation after a direct notification action.
        activity.intent?.let { HostForegroundService.handleNotificationStopIntent(it) }
    }

    override fun onNewIntent(intent: Intent) {
        // Existing Activity brought to the front from "Stop safely".
        HostForegroundService.handleNotificationStopIntent(intent)
    }

    @Command
    fun deviceTelemetry(invoke: Invoke) {
        val appContext = activity.applicationContext
        val result = JSObject().apply {
            put("platform", "android")
            put("source", "android-system-services")
        }

        runCatching {
            val activityManager =
                appContext.getSystemService(Context.ACTIVITY_SERVICE) as? ActivityManager
            activityManager?.let { manager ->
                val memory = ActivityManager.MemoryInfo()
                manager.getMemoryInfo(memory)
                if (memory.totalMem > 0L) {
                    val totalMemoryMib = memory.totalMem / BYTES_PER_MIB
                    val availableMemoryMib = memory.availMem
                        .coerceAtMost(memory.totalMem)
                        .coerceAtLeast(0L) / BYTES_PER_MIB
                    result.put("totalMemoryMib", totalMemoryMib)
                    result.put("availableMemoryMib", availableMemoryMib)
                }
            }
        }

        runCatching {
            appContext.registerReceiver(
                null,
                IntentFilter(Intent.ACTION_BATTERY_CHANGED)
            )
        }.getOrNull()?.let { battery ->
            val level = battery.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
            val scale = battery.getIntExtra(BatteryManager.EXTRA_SCALE, -1)
            if (level >= 0 && scale > 0) {
                result.put(
                    "batteryPercentage",
                    ((level * 100L) / scale).coerceIn(0L, 100L).toInt()
                )
            }

            when (battery.getIntExtra(BatteryManager.EXTRA_STATUS, BatteryManager.BATTERY_STATUS_UNKNOWN)) {
                BatteryManager.BATTERY_STATUS_CHARGING,
                BatteryManager.BATTERY_STATUS_FULL -> result.put("charging", true)
                BatteryManager.BATTERY_STATUS_DISCHARGING,
                BatteryManager.BATTERY_STATUS_NOT_CHARGING -> result.put("charging", false)
            }

            val temperatureTenthsCelsius =
                battery.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, Int.MIN_VALUE)
            if (temperatureTenthsCelsius != Int.MIN_VALUE) {
                result.put("batteryTemperatureCelsius", temperatureTenthsCelsius / 10.0)
            }
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            runCatching {
                appContext.getSystemService(Context.POWER_SERVICE) as? PowerManager
            }.getOrNull()?.let { manager ->
                runCatching { thermalStatusLabel(manager.currentThermalStatus) }
                    .getOrNull()
                    ?.let { status -> result.put("thermalStatus", status) }
            }
        }

        runCatching {
            StatFs(appContext.filesDir.absolutePath).availableBytes
        }.getOrNull()
            ?.takeIf { availableBytes -> availableBytes >= 0L }
            ?.let { availableBytes ->
                result.put("freeStorageMib", availableBytes / BYTES_PER_MIB)
            }

        invoke.resolve(result)
    }

    @Command
    fun startHosting(invoke: Invoke) {
        val args = invoke.parseArgs(StartHostingArgs::class.java)
        val label = args.label?.takeIf { it.isNotBlank() } ?: "Hosting a Slopity server"
        val activeServerCount = (args.activeServerCount ?: 1).coerceAtLeast(1)
        val requestId = HostForegroundService.registerStartRequest { error ->
            if (error != null) {
                invoke.reject(error)
            } else {
                invoke.resolve(status())
            }
        }
        if (requestId == null) {
            invoke.reject("Android foreground hosting is already starting or stopping. Wait for that request to finish before starting or updating hosting.")
            return
        }

        val intent = Intent(activity.applicationContext, HostForegroundService::class.java)
            .setAction(HostForegroundService.ACTION_START_OR_UPDATE)
            .putExtra(HostForegroundService.EXTRA_LABEL, label)
            .putExtra(HostForegroundService.EXTRA_ACTIVE_SERVER_COUNT, activeServerCount)
            .putExtra(HostForegroundService.EXTRA_START_REQUEST_ID, requestId)
        try {
            ContextCompat.startForegroundService(activity.applicationContext, intent)
        } catch (error: RuntimeException) {
            val detail = error.message?.takeIf { it.isNotBlank() }
                ?: error.javaClass.simpleName
            HostForegroundService.failStartRequest(
                requestId,
                "Android rejected the Slopity foreground-service start: $detail"
            )
        }

        // Do not resolve a Tauri start while Android has merely queued the service.
        // This also prevents indefinitely pending invokes if Android never delivers it.
        android.os.Handler(android.os.Looper.getMainLooper()).postDelayed({
            if (HostForegroundService.failStartRequest(
                    requestId,
                    "Android did not confirm foreground hosting within 15 seconds."
                ) && !HostForegroundService.snapshot().active
            ) {
                runCatching {
                    activity.applicationContext.stopService(
                        Intent(activity.applicationContext, HostForegroundService::class.java)
                    )
                }
            }
        }, START_CONFIRMATION_TIMEOUT_MS)
    }

    @Command
    fun stopHosting(invoke: Invoke) {
        HostForegroundService.markShutdownRequested()
        try {
            val stopRequested = activity.applicationContext.stopService(
                Intent(activity.applicationContext, HostForegroundService::class.java)
            )
            if (!stopRequested) {
                HostForegroundService.markStopped()
            }
            val reason = if (stopRequested) {
                "Android foreground host stop was requested; removal is confirmed only when onDestroy runs."
            } else {
                "Android foreground host service was not running."
            }
            invoke.resolve(status(reason))
        } catch (error: RuntimeException) {
            val detail = error.message?.takeIf { it.isNotBlank() }
                ?: error.javaClass.simpleName
            invoke.reject("Android rejected the Slopity foreground-service stop: $detail")
        }
    }

    @Command
    fun getStatus(invoke: Invoke) {
        invoke.resolve(status())
    }

    private fun thermalStatusLabel(status: Int): String? {
        return when (status) {
            PowerManager.THERMAL_STATUS_NONE -> "none"
            PowerManager.THERMAL_STATUS_LIGHT -> "light"
            PowerManager.THERMAL_STATUS_MODERATE -> "moderate"
            PowerManager.THERMAL_STATUS_SEVERE -> "severe"
            PowerManager.THERMAL_STATUS_CRITICAL -> "critical"
            PowerManager.THERMAL_STATUS_EMERGENCY -> "emergency"
            PowerManager.THERMAL_STATUS_SHUTDOWN -> "shutdown"
            else -> null
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
        val notificationManager = runCatching {
            activity.applicationContext.getSystemService(NotificationManager::class.java)
        }.getOrNull()
        val appNotificationsEnabled = runCatching {
            notificationManager?.areNotificationsEnabled() ?: false
        }.getOrDefault(false)
        // Ensure the channel exists before Rust's pre-start delivery check.
        // Re-creating an existing channel does not override a user's disabled setting.
        val channelReady = runCatching {
            HostForegroundService.ensureNotificationChannel(activity.applicationContext)
        }.isSuccess
        val channelEnabled = channelReady && runCatching {
            notificationManager?.getNotificationChannel(HostForegroundService.CHANNEL_ID)
                ?.importance?.let { it != NotificationManager.IMPORTANCE_NONE } ?: false
        }.getOrDefault(false)
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
                "Hosting cannot remain active because POST_NOTIFICATIONS is denied and Slopity requires visible foreground hosting."
            !appNotificationsEnabled ->
                "Hosting cannot remain active because application notifications are disabled in Android settings."
            !channelEnabled ->
                "Hosting cannot remain active because the Slopity hosting notification channel is disabled."
            notificationVisible ->
                "Android foreground hosting was acknowledged and its notification should be visible; device UI visibility has not been independently verified."
            else ->
                "Android foreground hosting is active, but notification visibility could not be confirmed."
        }

        return JSObject().apply {
            put("platform", "android")
            put("active", snapshot.active)
            put("startRequestPending", snapshot.startRequestPending)
            put("notificationVisible", notificationVisible)
            put("notificationPermissionGranted", notificationPermissionGranted)
            put("notificationPermissionRequired", notificationPermissionRequired)
            put("notificationsEnabled", appNotificationsEnabled)
            put("notificationChannelEnabled", channelEnabled)
            put("label", snapshot.label)
            put("activeServerCount", snapshot.activeServerCount)
            put("stopRequestPending", snapshot.stopRequestPending)
            put("reason", reason)
        }
    }

    companion object {
        private const val BYTES_PER_MIB = 1024L * 1024L
        private const val START_CONFIRMATION_TIMEOUT_MS = 15_000L
    }
}
