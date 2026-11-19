package net.mioyi.weftui

import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer

class OpReaderTest {
    private class Batch {
        private val out = ByteArrayOutputStream().apply { write(WeftProtocol.VERSION) }

        fun byte(value: Int) = apply { out.write(value) }

        fun u32(vararg values: Int) = apply {
            for (value in values) for (i in 0 until 4) out.write(value ushr (8 * i))
        }

        fun ops(): List<Op> = OpReader.readAll(ByteBuffer.wrap(out.toByteArray()))
    }

    @Test
    fun anEmptyBatchHasNoOps() {
        assertNull(OpReader(ByteBuffer.wrap(byteArrayOf(1))).next())
    }

    @Test
    fun rejectsTruncatedString() {
        assertThrows(OpException::class.java) { Batch().byte(2).u32(1, 0).byte(3).u32(100).ops() }
    }
}
