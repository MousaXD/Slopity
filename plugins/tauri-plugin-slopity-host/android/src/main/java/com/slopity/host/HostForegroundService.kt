package com.slopity.host

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import java.util.UUID

internal data class HostServiceSnapshot(
    val active: Boolean = false,
    val startRequestPending: Boolean = false,
    val stopRequestPending: Boolean = false,
    val label: String = "",
    val activeServerCount: Int = 0
)

class HostForegroundService : Service() {
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_REQUEST_STOP) {
            markStopRequested()
            val current = snapshot()
            if (current.active) {
                // A failed notification refresh must not crash the Rust hosting process.
                runCatching {
                    getSystemService(NotificationManager::class.java).notify(
                        NOTIFICATION_ID,
                        buildNotification(current)
                    )
                }
            }
            openSlopityForStopReconciliation()
            return START_NOT_STICKY
        }

        val requestId = intent?.getIntExtra(EXTRA_START_REQUEST_ID, -1) ?: -1
        if (!hasStartRequest(requestId)) {
            // A stop/timeout may have canceled this queued start before delivery.
            if (!snapshot().active) stopSelf(startId)
            return START_NOT_STICKY
        }

        val label = intent?.getStringExtra(EXTRA_LABEL)
            ?.takeIf { it.isNotBlank() }
            ?: "Hosting a Slopity server"
        val activeServerCount = intent
            ?.getIntExtra(EXTRA_ACTIVE_SERVER_COUNT, 1)
            ?.coerceAtLeast(1)
            ?: 1

        try {
            ensureNotificationChannel(this)
            val notification = buildNotification(
                snapshot().copy(label = label, activeServerCount = activeServerCount)
            )
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }

            // Only a successful startForeground() may acknowledge hosting to Rust.
            if (!finishStartRequest(requestId, label, activeServerCount) && !snapshot().active) {
                stopSelf(startId)
            }
        } catch (error: RuntimeException) {
            val detail = error.message?.takeIf { it.isNotBlank() }
                ?: error.javaClass.simpleName
            markStartFailed("Android failed to promote Slopity into foreground hosting: $detail")
            stopSelf(startId)
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        markStartFailed("Android destroyed the Slopity foreground host service.")
        stopForeground(STOP_FOREGROUND_REMOVE)
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun openSlopityForStopReconciliation() {
        // Android may restrict activity launches from services. This is a best-effort
        // prompt to reconcile, not proof that Rust received the stop request.
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
                    // Android 12+ blocks notification trampolines from a service
                    // into an Activity. The notification must open the Activity
                    // directly, where HostPlugin records the one-time stop token.
                    packageManager.getLaunchIntentForPackage(packageName)?.let { stopIntent ->
                        stopIntent.action = ACTION_REQUEST_STOP
                        stopIntent.putExtra(EXTRA_STOP_TOKEN, currentStopToken())
                        stopIntent.addFlags(
                            Intent.FLAG_ACTIVITY_NEW_TASK or
                                Intent.FLAG_ACTIVITY_CLEAR_TOP or
                                Intent.FLAG_ACTIVITY_SINGLE_TOP
                        )
                        val stopPendingIntent = PendingIntent.getActivity(
                            this@HostForegroundService,
                            1,
                            stopIntent,
                            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
                        )
                        addAction(0, "Stop safely", stopPendingIntent)
                    }
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
            "Stop requested. Open Slopity to finish shutting down hosted servers safely."
        snapshot.label.isNotBlank() -> snapshot.label
        else -> "Hosting a Slopity server"
    }

    companion object {
        const val ACTION_START_OR_UPDATE = "com.slopity.host.action.START_OR_UPDATE"
        const val ACTION_REQUEST_STOP = "com.slopity.host.action.REQUEST_STOP"
        const val EXTRA_LABEL = "slopity.host.label"
        const val EXTRA_ACTIVE_SERVER_COUNT = "slopity.host.activeServerCount"
        const val EXTRA_START_REQUEST_ID = "slopity.host.startRequestId"
        const val EXTRA_STOP_TOKEN = "slopity.host.stopToken"
        internal const val CHANNEL_ID = "slopity_hosting"
        private const val NOTIFICATION_ID = 8_081

        private val stateLock = Any()
        private var state = HostServiceSnapshot()
        private var nextRequestId = 1
        private val pendingStarts = mutableMapOf<Int, (String?) -> Unit>()
        private var stopToken = UUID.randomUUID().toString()

        internal fun ensureNotificationChannel(context: Context) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Slopity hosting",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows when user-started Slopity server hosting is active."
                setShowBadge(false)
            }
            context.getSystemService(NotificationManager::class.java)
                .createNotificationChannel(channel)
        }

        private fun currentStopToken(): String = synchronized(stateLock) { stopToken }

        internal fun handleNotificationStopIntent(intent: Intent): Boolean {
            if (intent.action != ACTION_REQUEST_STOP) return false
            val receivedToken = intent.getStringExtra(EXTRA_STOP_TOKEN) ?: return false
            synchronized(stateLock) {
                if (receivedToken != stopToken || !state.active || state.stopRequestPending) {
                    return false
                }
                // Reject replays or an external app forging ACTION_REQUEST_STOP.
                stopToken = UUID.randomUUID().toString()
                state = state.copy(stopRequestPending = true)
            }
            failOutstandingStartRequests(
                "A notification stop request is pending; foreground host updates are rejected."
            )
            return true
        }

        internal fun snapshot(): HostServiceSnapshot = synchronized(stateLock) { state.copy() }

        internal fun registerStartRequest(callback: (String?) -> Unit): Int? =
            synchronized(stateLock) {
                // Only one foreground promotion may be in flight. Concurrent
                // callers otherwise observe another request's pending flag or
                // apply a stale server-count update out of order.
                if (state.stopRequestPending || pendingStarts.isNotEmpty()) {
                    null
                } else {
                    val requestId = nextRequestId++
                    pendingStarts[requestId] = callback
                    state = state.copy(startRequestPending = true)
                    requestId
                }
            }

        internal fun hasStartRequest(requestId: Int): Boolean = synchronized(stateLock) {
            !state.stopRequestPending && pendingStarts.containsKey(requestId)
        }

        internal fun finishStartRequest(
            requestId: Int,
            label: String,
            activeServerCount: Int
        ): Boolean {
            var succeeded = false
            val callback = synchronized(stateLock) {
                val registered = pendingStarts.remove(requestId)
                if (registered != null && !state.stopRequestPending) {
                    state = state.copy(
                        active = true,
                        startRequestPending = pendingStarts.isNotEmpty(),
                        label = label,
                        activeServerCount = activeServerCount
                    )
                    succeeded = true
                } else {
                    state = state.copy(startRequestPending = pendingStarts.isNotEmpty())
                }
                registered
            }
            callback?.invoke(
                if (succeeded) null
                else "The Slopity host start was canceled by a pending stop request."
            )
            return succeeded
        }

        internal fun failStartRequest(requestId: Int, reason: String): Boolean {
            val callback = synchronized(stateLock) {
                val registered = pendingStarts.remove(requestId)
                state = state.copy(startRequestPending = pendingStarts.isNotEmpty())
                registered
            }
            callback?.invoke(reason)
            return callback != null
        }

        internal fun failOutstandingStartRequests(reason: String) {
            val callbacks = synchronized(stateLock) {
                val registered = pendingStarts.values.toList()
                pendingStarts.clear()
                state = state.copy(startRequestPending = false)
                registered
            }
            callbacks.forEach { callback -> callback(reason) }
        }

        private fun markStopRequested() {
            synchronized(stateLock) {
                if (state.active) {
                    state = state.copy(stopRequestPending = true)
                }
            }
            failOutstandingStartRequests(
                "A notification stop request is pending; foreground host updates are rejected."
            )
        }

        internal fun markStartFailed(reason: String) {
            val callbacks = synchronized(stateLock) {
                val registered = pendingStarts.values.toList()
                pendingStarts.clear()
                state = HostServiceSnapshot()
                stopToken = UUID.randomUUID().toString()
                registered
            }
            callbacks.forEach { callback -> callback(reason) }
        }

        internal fun markShutdownRequested() {
            // Prevent any new or already-queued start from overtaking shutdown.
            synchronized(stateLock) {
                state = state.copy(stopRequestPending = true)
            }
            failOutstandingStartRequests(
                "Android host shutdown canceled pending foreground starts."
            )
        }

        internal fun markStopped() {
            markStartFailed("Android foreground hosting was stopped.")
        }
    }
}
