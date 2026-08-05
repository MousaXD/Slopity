package com.pockethost.app.data

import com.pockethost.app.domain.ServerProfile
import com.pockethost.app.domain.ServerRuntime

class InMemoryServerProfileRepository : ServerProfileRepository {
    private val profiles = listOf(
        ServerProfile(
            id = "paper-demo",
            displayName = "Minecraft Paper",
            runtime = ServerRuntime.JVM,
            packageId = "paper-uninstalled",
            memoryMb = 3_072,
            ports = listOf(25_565),
            description = "Architecture example only. JVM and Paper installation are not implemented.",
        ),
        ServerProfile(
            id = "node-demo",
            displayName = "Node.js service",
            runtime = ServerRuntime.NODE_JS,
            packageId = "node-uninstalled",
            memoryMb = 512,
            ports = listOf(3_000),
            description = "Example of a second runtime family. Node.js is not bundled.",
        ),
        ServerProfile(
            id = "pocketmine-demo",
            displayName = "PocketMine-MP",
            runtime = ServerRuntime.PHP,
            packageId = "pocketmine-uninstalled",
            memoryMb = 1_024,
            ports = listOf(19_132),
            description = "Example Bedrock-oriented PHP workload. PHP is not bundled.",
        ),
    )

    override fun list(): List<ServerProfile> = profiles

    override fun get(id: String): ServerProfile? = profiles.firstOrNull { it.id == id }
}
