package com.pockethost.app.runtime

import com.pockethost.app.domain.ServerRuntime

class RuntimeRegistry(adapters: List<RuntimeAdapter>) {
    private val adaptersByRuntime = adapters.associateBy(RuntimeAdapter::runtime)

    fun adapterFor(runtime: ServerRuntime): RuntimeAdapter? = adaptersByRuntime[runtime]

    fun all(): List<RuntimeAdapter> = adaptersByRuntime.values.sortedBy { it.runtime.name }

    companion object {
        fun foundationDefaults(): RuntimeRegistry = RuntimeRegistry(
            ServerRuntime.entries.map { runtime ->
                UnsupportedRuntimeAdapter(
                    runtime = runtime,
                    explanation = "${runtime.name} runtime adapter is planned but not installed.",
                )
            },
        )
    }
}
