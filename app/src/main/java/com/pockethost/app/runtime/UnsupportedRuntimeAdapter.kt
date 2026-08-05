package com.pockethost.app.runtime

import com.pockethost.app.domain.RuntimeAvailability
import com.pockethost.app.domain.ServerProfile
import com.pockethost.app.domain.ServerRuntime

class UnsupportedRuntimeAdapter(
    override val runtime: ServerRuntime,
    private val explanation: String,
) : RuntimeAdapter {
    override fun availability(): RuntimeAvailability = RuntimeAvailability(
        available = false,
        reason = explanation,
    )

    override fun start(profile: ServerProfile): Result<Unit> =
        Result.failure(UnsupportedOperationException(explanation))

    override fun stop(profileId: String): Result<Unit> =
        Result.failure(UnsupportedOperationException(explanation))
}
