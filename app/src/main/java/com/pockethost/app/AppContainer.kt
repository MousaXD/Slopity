package com.pockethost.app

import android.content.Context
import com.pockethost.app.capability.DeviceCapabilityProbe
import com.pockethost.app.data.InMemoryServerProfileRepository
import com.pockethost.app.runtime.RuntimeRegistry
import com.pockethost.app.runtime.ServerOrchestrator

class AppContainer(context: Context) {
    private val applicationContext = context.applicationContext

    val capabilityProbe = DeviceCapabilityProbe(applicationContext)
    val profileRepository = InMemoryServerProfileRepository()
    val runtimeRegistry = RuntimeRegistry.foundationDefaults()
    val orchestrator = ServerOrchestrator(
        capabilityProvider = capabilityProbe::read,
        runtimeRegistry = runtimeRegistry,
    )
}
