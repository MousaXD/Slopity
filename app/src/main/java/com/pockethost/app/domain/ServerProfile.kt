package com.pockethost.app.domain

data class ServerProfile(
    val id: String,
    val displayName: String,
    val runtime: ServerRuntime,
    val packageId: String,
    val memoryMb: Int,
    val ports: List<Int>,
    val arguments: List<String> = emptyList(),
    val autoRestart: Boolean = false,
    val description: String = "",
)

enum class ServerRuntime {
    JVM,
    PHP,
    NODE_JS,
    PYTHON,
    NATIVE,
    CUSTOM,
}

enum class ServerStatus {
    STOPPED,
    STARTING,
    RUNNING,
    STOPPING,
    FAILED,
    UNSUPPORTED,
}
