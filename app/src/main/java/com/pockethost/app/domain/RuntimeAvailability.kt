package com.pockethost.app.domain

data class RuntimeAvailability(
    val available: Boolean,
    val reason: String,
)

data class StartDecision(
    val allowed: Boolean,
    val reasons: List<String>,
) {
    val summary: String
        get() = if (allowed) {
            "Preflight passed"
        } else {
            reasons.joinToString(separator = " ")
        }
}
