package com.apex.kubemanager.mobile.core.triage

/**
 * Health of a single object as decided by a triage rule.
 * [wireName] values are shared with iOS and desktop (docs/mobile/architecture.md, section 3).
 */
enum class VerdictStatus(val wireName: String) {
    OK("ok"),
    WARN("warn"),
    FAIL("fail"),
    UNKNOWN("unknown"),
}

/** Output of a triage rule. Field names match the shared spec; [since] is epoch millis. */
data class Verdict(
    val status: VerdictStatus,
    val reason: String,
    val ruleId: String,
    val since: Long? = null,
)
