package dev.weft

import java.nio.BufferUnderflowException
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** One view protocol op (docs/src/content/docs/protocol.md, "Ops"). */
internal sealed interface Op {
    data class Create(val scope: Int, val template: Int) : Op

    data class Set(val scope: Int, val slot: Int, val value: WeftValue) : Op

    data class Branch(val scope: Int, val point: Int, val child: Int) : Op

    class Rows(val scope: Int, val point: Int, val children: IntArray) : Op {
        override fun equals(other: Any?): Boolean =
            other is Rows && scope == other.scope && point == other.point && children.contentEquals(other.children)

        override fun hashCode(): Int = (scope * 31 + point) * 31 + children.contentHashCode()

        override fun toString(): String = "Rows(scope=$scope, point=$point, children=${children.contentToString()})"
    }

    data class Attach(val scope: Int, val point: Int, val child: Int) : Op
}

/** Why a batch of ops could not be decoded. */
internal class OpException(message: String) : Exception(message)

/**
 * Decodes one batch of ops in the binary encoding (version 1). Reads [bytes]
 * from its position to its limit; strings are copied, so decoded ops outlive
 * the buffer.
 */
internal class OpReader(bytes: ByteBuffer) {
    private val bytes: ByteBuffer = bytes.slice().order(ByteOrder.LITTLE_ENDIAN)

    init {
        if (!this.bytes.hasRemaining()) throw OpException("truncated")
        val version = this.bytes.get().toInt() and 0xff
        if (version != VERSION) throw OpException("version $version")
    }

    /** The next op, or null at the end of the batch. */
    fun next(): Op? {
        if (!bytes.hasRemaining()) return null
        try {
            return when (val opcode = byte()) {
                1 -> Op.Create(u32(), u32())
                2 -> Op.Set(u32(), u32(), value())
                3 -> Op.Branch(u32(), u32(), u32())
                4 -> {
                    val scope = u32()
                    val point = u32()
                    val count = bytes.int.toUInt().toLong()
                    if (count > bytes.remaining() / 4) throw OpException("truncated")
                    Op.Rows(scope, point, IntArray(count.toInt()) { u32() })
                }
                5 -> Op.Attach(u32(), u32(), u32())
                else -> throw OpException("opcode $opcode")
            }
        } catch (_: BufferUnderflowException) {
            throw OpException("truncated")
        }
    }

    private fun byte(): Int = bytes.get().toInt() and 0xff

    /** A u32 scope, slot, point or template; real ones fit an Int. */
    private fun u32(): Int = bytes.int

    private fun value(): WeftValue = when (val tag = byte()) {
        0 -> WeftValue.Bool(false)
        1 -> WeftValue.Bool(true)
        2 -> WeftValue.Number(bytes.double)
        3 -> {
            val length = bytes.int.toUInt().toLong()
            if (length > bytes.remaining()) throw OpException("truncated")
            val utf8 = ByteArray(length.toInt())
            bytes.get(utf8)
            WeftValue.Text(utf8.decodeToString())
        }
        else -> throw OpException("tag $tag")
    }

    companion object {
        /** The encoding version this reader decodes. */
        const val VERSION = 1
    }
}
