package com.slopity.host

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class HostForegroundServiceTest {
    @Before
    fun resetSnapshot() {
        HostForegroundService.markStopped()
    }

    @Test
    fun startRemainsPendingUntilForegroundPromotionIsAcknowledged() {
        val results = mutableListOf<String?>()
        val requestId = HostForegroundService.registerStartRequest { results.add(it) }
        assertNotNull(requestId)
        assertFalse(HostForegroundService.snapshot().active)
        assertTrue(HostForegroundService.snapshot().startRequestPending)
        assertTrue(HostForegroundService.hasStartRequest(requestId!!))
        assertTrue(results.isEmpty())

        assertTrue(HostForegroundService.finishStartRequest(requestId, "Two servers", 2))
        val snapshot = HostForegroundService.snapshot()
        assertTrue(snapshot.active)
        assertFalse(snapshot.startRequestPending)
        assertEquals(2, snapshot.activeServerCount)
        assertEquals("Two servers", snapshot.label)
        assertEquals(listOf<String?>(null), results)
    }

    @Test
    fun shutdownRejectsPendingAndDisallowsStaleQueuedStarts() {
        val results = mutableListOf<String?>()
        val requestId = HostForegroundService.registerStartRequest { results.add(it) }!!
        HostForegroundService.markShutdownRequested()

        assertTrue(HostForegroundService.snapshot().stopRequestPending)
        assertFalse(HostForegroundService.hasStartRequest(requestId))
        assertFalse(HostForegroundService.finishStartRequest(requestId, "Stale", 3))
        assertNull(HostForegroundService.registerStartRequest { })
        assertEquals(1, results.size)
        assertNotNull(results.single())

        HostForegroundService.markStopped()
        assertFalse(HostForegroundService.snapshot().stopRequestPending)
        assertNotNull(HostForegroundService.registerStartRequest { })
    }

    @Test
    fun failedUpdatePreservesPreviouslyAcknowledgedForegroundState() {
        val first = HostForegroundService.registerStartRequest { }!!
        assertTrue(HostForegroundService.finishStartRequest(first, "One server", 1))
        val secondResult = mutableListOf<String?>()
        val second = HostForegroundService.registerStartRequest { secondResult.add(it) }!!

        assertTrue(HostForegroundService.failStartRequest(second, "start rejected"))
        val snapshot = HostForegroundService.snapshot()
        assertTrue(snapshot.active)
        assertFalse(snapshot.startRequestPending)
        assertEquals(1, snapshot.activeServerCount)
        assertEquals("One server", snapshot.label)
        assertEquals(listOf("start rejected"), secondResult)
    }

    @Test
    fun overlappingStartsAreRejectedUntilFirstPromotionCompletes() {
        val firstResult = mutableListOf<String?>()
        val first = HostForegroundService.registerStartRequest { firstResult.add(it) }!!
        assertNull(HostForegroundService.registerStartRequest { })
        assertTrue(HostForegroundService.hasStartRequest(first))
        assertTrue(HostForegroundService.finishStartRequest(first, "First", 1))
        assertEquals(listOf<String?>(null), firstResult)

        val next = HostForegroundService.registerStartRequest { }!!
        assertTrue(HostForegroundService.finishStartRequest(next, "Two", 2))
        assertEquals(2, HostForegroundService.snapshot().activeServerCount)
        assertFalse(HostForegroundService.snapshot().startRequestPending)
    }

    @Test
    fun failedPromotionRejectsPendingAndResetsNativeState() {
        val results = mutableListOf<String?>()
        val first = HostForegroundService.registerStartRequest { results.add(it) }!!
        assertNull(HostForegroundService.registerStartRequest { })
        HostForegroundService.markStartFailed("foreground promotion failed")

        assertFalse(HostForegroundService.hasStartRequest(first))
        assertEquals(listOf("foreground promotion failed"), results)
        assertFalse(HostForegroundService.snapshot().active)
        assertFalse(HostForegroundService.snapshot().startRequestPending)
        assertEquals(0, HostForegroundService.snapshot().activeServerCount)
    }
}
