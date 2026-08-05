package com.pockethost.app.service

import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat

object HostServiceController {
    fun start(context: Context) {
        val intent = Intent(context, ServerHostService::class.java)
            .setAction(ServerHostService.ACTION_START)
        ContextCompat.startForegroundService(context, intent)
    }

    fun stop(context: Context) {
        val intent = Intent(context, ServerHostService::class.java)
            .setAction(ServerHostService.ACTION_STOP)
        context.startService(intent)
    }
}
