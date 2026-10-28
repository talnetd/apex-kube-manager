package com.apex.kubemanager.mobile.core.triage

import org.junit.Assert.assertEquals
import org.junit.Test

class VerdictTest {
    @Test
    fun statusWireNamesMatchSpec() {
        assertEquals(listOf("ok", "warn", "fail", "unknown"), VerdictStatus.entries.map { it.wireName })
    }
}
