package com.pockethost.app.domain

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class CapabilityAdvisorTest {
    @Test
    fun reservesMemoryForAndroidAndCapsConcurrency() {
        val plan = CapabilityAdvisor.advise(
            DeviceCapabilities(
                totalMemoryMb = 8_192,
                availableMemoryMb = 6_000,
                cpuCores = 8,
                supportedAbis = listOf("arm64-v8a"),
                freeStorageMb = 20_000,
                thermalStatus = ThermalStatus.NONE,
            ),
        )

        assertEquals(2_048, plan.reservedForAndroidMb)
        assertEquals(6_000, plan.usableForServersMb)
        assertEquals(6, plan.recommendedMaxConcurrentServers)
        assertEquals(1_000, plan.recommendedMemoryPerServerMb)
    }

    @Test
    fun blocksRecommendationWhenResourcesAreTooLow() {
        val plan = CapabilityAdvisor.advise(
            DeviceCapabilities(
                totalMemoryMb = 2_048,
                availableMemoryMb = 256,
                cpuCores = 2,
                supportedAbis = listOf("armeabi-v7a"),
                freeStorageMb = 1_000,
                thermalStatus = ThermalStatus.SEVERE,
            ),
        )

        assertEquals(0, plan.recommendedMaxConcurrentServers)
        assertTrue(plan.warnings.size >= 3)
    }
}
