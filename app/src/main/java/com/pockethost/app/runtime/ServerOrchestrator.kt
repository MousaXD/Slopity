package com.pockethost.app.runtime

import com.pockethost.app.domain.CapabilityAdvisor
import com.pockethost.app.domain.DeviceCapabilities
import com.pockethost.app.domain.ServerProfile
import com.pockethost.app.domain.StartDecision

class ServerOrchestrator(
    private val capabilityProvider: () -> DeviceCapabilities,
    private val runtimeRegistry: RuntimeRegistry,
) {
    fun preflight(profile: ServerProfile): StartDecision {
        val capabilities = capabilityProvider()
        val plan = CapabilityAdvisor.advise(capabilities)
        val adapter = runtimeRegistry.adapterFor(profile.runtime)
        val reasons = buildList {
            if (profile.memoryMb <= 0) add("Memory allocation must be positive.")
            if (profile.memoryMb > plan.usableForServersMb) {
                add(
                    "Profile requests ${profile.memoryMb} MB, but the current conservative budget is " +
                        "${plan.usableForServersMb} MB.",
                )
            }
            if (profile.ports.isEmpty()) add("At least one port must be declared.")
            if (profile.ports.any { it !in 1..65_535 }) add("A declared port is outside 1-65535.")
            if (profile.ports.distinct().size != profile.ports.size) add("Duplicate ports are not allowed.")
            if (adapter == null) {
                add("No adapter is registered for ${profile.runtime}.")
            } else {
                val availability = adapter.availability()
                if (!availability.available) add(availability.reason)
            }
            addAll(plan.warnings)
        }

        return StartDecision(
            allowed = reasons.isEmpty(),
            reasons = reasons,
        )
    }
}
