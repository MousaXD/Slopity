package com.pockethost.app.runtime

import com.pockethost.app.domain.RuntimeAvailability
import com.pockethost.app.domain.ServerProfile
import com.pockethost.app.domain.ServerRuntime

interface RuntimeAdapter {
    val runtime: ServerRuntime

    fun availability(): RuntimeAvailability

    fun start(profile: ServerProfile): Result<Unit>

    fun stop(profileId: String): Result<Unit>
}
