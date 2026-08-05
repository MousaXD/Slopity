package com.slopity.host

import android.app.Activity
import android.content.Intent
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class StartHostingArgs {
    var label: String? = null
}

@TauriPlugin
class HostPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun startHosting(invoke: Invoke) {
        val args = invoke.parseArgs(StartHostingArgs::class.java)
        val label = args.label?.takeIf { it.isNotBlank() } ?: "Hosting a Slopity server"
        val intent = Intent(activity.applicationContext, HostForegroundService::class.java)
            .putExtra(HostForegroundService.EXTRA_LABEL, label)
        ContextCompat.startForegroundService(activity.applicationContext, intent)
        invoke.resolve(status(active = true, notificationVisible = true, reason = label))
    }

    @Command
    fun stopHosting(invoke: Invoke) {
        val intent = Intent(activity.applicationContext, HostForegroundService::class.java)
        activity.applicationContext.stopService(intent)
        invoke.resolve(
            status(
                active = false,
                notificationVisible = false,
                reason = "Android foreground host service stopped."
            )
        )
    }

    private fun status(
        active: Boolean,
        notificationVisible: Boolean,
        reason: String
    ): JSObject {
        return JSObject().apply {
            put("platform", "android")
            put("active", active)
            put("notificationVisible", notificationVisible)
            put("reason", reason)
        }
    }
}
