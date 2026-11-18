package dev.weft

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer

class OpReaderTest {
    private class Batch {
        private val out = ByteArrayOutputStream().apply { write(OpReader.VERSION) }

        fun byte(value: Int) = apply { out.write(value) }

        fun u32(vararg values: Int) = apply {
            for (value in values) for (i in 0 until 4) out.write(value ushr (8 * i))
        }

        fun value(value: WeftValue) = apply { out.write(value.encode()) }

        fun ops(): List<Op> {
            val reader = OpReader(ByteBuffer.wrap(out.toByteArray()))
            return generateSequence { reader.next() }.toList()
        }
    }

    @Test
    fun decodesEveryOp() {
        val ops = Batch()
            .byte(1).u32(3, 2)
            .byte(2).u32(3, 0).value(WeftValue.Text("milk 🥛"))
            .byte(2).u32(3, 1).value(WeftValue.Number(12.5))
            .byte(2).u32(3, 2).value(WeftValue.Bool(true))
            .byte(4).u32(2, 0, 3, 3, 4, 5)
            .byte(3).u32(1, 0, 0)
            .byte(5).u32(0, 0, 1)
            .ops()
        assertEquals(
            listOf(
                Op.Create(3, 2),
                Op.Set(3, 0, WeftValue.Text("milk 🥛")),
                Op.Set(3, 1, WeftValue.Number(12.5)),
                Op.Set(3, 2, WeftValue.Bool(true)),
                Op.Rows(2, 0, intArrayOf(3, 4, 5)),
                Op.Branch(1, 0, 0),
                Op.Attach(0, 0, 1),
            ),
            ops,
        )
    }

    @Test
    fun anEmptyBatchHasNoOps() {
        assertNull(OpReader(ByteBuffer.wrap(byteArrayOf(1))).next())
    }

    @Test
    fun rejectsMalformedBatches() {
        assertThrows(OpException::class.java) { OpReader(ByteBuffer.wrap(byteArrayOf(2))) }
        assertThrows(OpException::class.java) { Batch().byte(9).ops() }
        assertThrows(OpException::class.java) { Batch().byte(1).u32(3).ops() }
        assertThrows(OpException::class.java) { Batch().byte(2).u32(1, 0).byte(7).ops() }
        // A row count beyond the batch is not allocated.
        assertThrows(OpException::class.java) { Batch().byte(4).u32(1, 0, -1).ops() }
        assertThrows(OpException::class.java) { Batch().byte(2).u32(1, 0).byte(3).u32(100).ops() }
    }
}
