package com.pockethost.app.data

import com.pockethost.app.domain.ServerProfile

interface ServerProfileRepository {
    fun list(): List<ServerProfile>
    fun get(id: String): ServerProfile?
}
