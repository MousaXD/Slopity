package com.slopity.host

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

internal data class HostServiceSnapshot(
    val active: Boolean = false,
    val startRequestPending: Boolean = false,
    val stopRequestPending: Boolean = false,
    val label: String = "",
    val activeServerCount: Int = 0
)

class HostForegroundService : Service() {
    override fun onCreate() {
        super.onCreate()
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Slopity hosting",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "Shows when user-started Slopity server hosting is active."
            setShowBadge(false)
        }
        getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_REQUEST_STOP) {
            markStopRequested()
            val current = snapshot()
            if (current.active) {
                getSystemService(NotificationManager::class.java).notify(
                    NOTIFICATION_ID,
                    buildNotification(current)
                )
            }
            openSlopityForStopReconciliation()
            return START_NOT_STICKY
        }

        val label = intent?.getStringExtra(EXTRA_LABEL)
            ?.takeIf { it.isNotBlank() }
            ?: "Hosting a Slopity server"
        val activeServerCount = intent
            ?.getIntExtra(EXTRA_ACTIVE_SERVER_COUNT, 1)
            ?.coerceAtLeast(1)
            ?: 1

        return try {
            markActive(label, activeServerCount)
            val notification = buildNotification(snapshot())
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            START_NOT_STICKY
        } catch (error: RuntimeException) {
            markStartFailed()
            stopSelf(startId)
            throw error
        }
    }

    override fun onDestroy() {
        markStopped()
        stopForeground(STOP_FOREGROUND_REMOVE)
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun openSlopityForStopReconciliation() {
        runCatching {
            packageManager.getLaunchIntentForPackage(packageName)?.let { launchIntent ->
                launchIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
                startActivity(launchIntent)
            }
        }
    }

    private fun buildNotification(snapshot: HostServiceSnapshot) =
        NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_sys_upload)
            .setContentTitle(notificationTitle(snapshot.activeServerCount))
            .setContentText(notificationText(snapshot))
            .setStyle(NotificationCompat.BigTextStyle().bigText(notificationText(snapshot)))
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .apply {
                packageManager.getLaunchIntentForPackage(packageName)?.let { launchIntent ->
                    val contentIntent = PendingIntent.getActivity(
                        this@HostForegroundService,
                        0,
                        launchIntent,
                        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
                    )
                    setContentIntent(contentIntent)
                }

                if (!snapshot.stopRequestPending) {
                    val stopIntent = Intent(this@HostForegroundService, HostForegroundService::class.java)
                        .setAction(ACTION_REQUEST_STOP)
                    val stopPendingIntent = PendingIntent.getService(
                        this@HostForegroundService,
                        1,
                        stopIntent,
                        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
                    )
                    addAction(0, "Stop safely", stopPendingIntent)
                }
            }
            .build()

    private fun notificationTitle(activeServerCount: Int): String = when (activeServerCount) {
        1 -> "Slopity server active"
        in 2..Int.MAX_VALUE -> "Slopity hosting $activeServerCount servers"
        else -> "Slopity hosting"
    }

    private fun notificationText(snapshot: HostServiceSnapshot): String = when {
        snapshot.stopRequestPending ->
            "Stop requested. Slopity is opening to shut down hosted servers safely."
        snapshot.label.isNotBlank() -> snapshot.label
        else -> "Hosting a Slopity server"
    }

    companion object {
        const val ACTION_START_OR_UPDATE = "com.slopity.host.action.START_OR_UPDATE"
        const val ACTION_REQUEST_STOP = "com.slopity.host.action.REQUEST_STOP"
        const val EXTRA_LABEL = "slopity.host.label"
        const val EXTRA_ACTIVE_SERVER_COUNT = "slopity.host.activeServerCount"
        internal const val CHANNEL_ID = "slopity_hosting"
        private const val NOTIFICATION_ID = 8_081

        private val stateLock = Any()
        private var state = HostServiceSnapshot()

        internal fun snapshot(): HostServiceSnapshot = synchronized(stateLock) { state.copy() }

        internal fun markStartRequested(label: String, activeServerCount: Int) {
            synchronized(stateLock) {
                state = state.copy(
                    startRequestPending = true,
                    stopRequestPending = false,
                    label = label,
                    activeServerCount = activeServerCount
                )
            }
        }

        private fun markActive(label: String, activeServerCount: Int) {
            synchronized(stateLock) {
                state = HostServiceSnapshot(
                    active = true,
                    label = label,
                    activeServerCount = activeServerCount
                )
            }
        }

        private fun markStopRequested() {
            synchronized(stateLock) {
                if (state.active) {
                    state = state.copy(stopRequestPending = true, startRequestPending = false)
                }
            }
        }

        internal fun markStartFailed() {
            synchronized(stateLock) {
                state = HostServiceSnapshot()
            }
        }

        internal fun markStopped() {
            synchronized(stateLock) {
                state = HostServiceSnapshot()
            }
        }
    }
}
