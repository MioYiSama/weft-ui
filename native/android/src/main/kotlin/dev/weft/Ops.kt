package dev.weft

import java.nio.Buffer
import java.nio.BufferUnderflowException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.charset.CharacterCodingException
import java.nio.charset.CharsetDecoder
import java.nio.charset.CodingErrorAction
import java.nio.charset.StandardCharsets

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
internal class OpException(val kind: String, message: String = kind) : Exception(message)

/**
 * Decodes one batch of ops in the binary encoding (version 1). Reads [bytes]
 * from its position to its limit; decoded ops outlive the buffer.
 */
internal class OpReader(bytes: ByteBuffer) {
    private val bytes: ByteBuffer = bytes.slice().order(ByteOrder.LITTLE_ENDIAN)
    private var utf8Decoder: CharsetDecoder? = null

    init {
        if (!this.bytes.hasRemaining()) throw OpException("truncated")
        val version = this.bytes.get().toInt() and 0xff
        if (version != WeftProtocol.VERSION) throw OpException("version", "version $version")
    }

    /** The next op, or null at the end of the batch. */
    fun next(): Op? {
        if (!bytes.hasRemaining()) return null
        try {
            return when (val opcode = byte()) {
                OpCode.CREATE -> Op.Create(u32(), u32())
                OpCode.SET -> Op.Set(u32(), u32(), value())
                OpCode.BRANCH -> Op.Branch(u32(), u32(), u32())
                OpCode.ROWS -> {
                    val scope = u32()
                    val point = u32()
                    val count = bytes.int.toUInt().toLong()
                    if (count > bytes.remaining() / 4) throw OpException("truncated")
                    Op.Rows(scope, point, IntArray(count.toInt()) { u32() })
                }
                OpCode.ATTACH -> Op.Attach(u32(), u32(), u32())
                else -> throw OpException("opcode", "opcode $opcode")
            }
        } catch (_: BufferUnderflowException) {
            throw OpException("truncated")
        }
    }

    private fun byte(): Int = bytes.get().toInt() and 0xff

    /** A u32 scope, slot, point or template; real ones fit an Int. */
    private fun u32(): Int = bytes.int

    private fun value(): WeftValue = when (val tag = byte()) {
        ValueTag.FALSE -> WeftValue.Bool(false)
        ValueTag.TRUE -> WeftValue.Bool(true)
        ValueTag.NUMBER -> WeftValue.Number(bytes.double)
        ValueTag.STRING -> WeftValue.Text(text())
        else -> throw OpException("tag", "tag $tag")
    }

    private fun text(): String {
        val length = bytes.int.toUInt().toLong()
        if (length > bytes.remaining()) throw OpException("truncated")
        if (length == 0L) return ""

        val decoder = utf8Decoder ?: StandardCharsets.UTF_8.newDecoder()
            .onMalformedInput(CodingErrorAction.REPORT)
            .onUnmappableCharacter(CodingErrorAction.REPORT)
            .also { utf8Decoder = it }
        // Bound decoding to the string without copying its bytes. Use Buffer's
        // limit setter for Android versions that lack the covariant override.
        val buffer: Buffer = bytes
        val limit = buffer.limit()
        buffer.limit(buffer.position() + length.toInt())
        try {
            return decoder.decode(bytes).toString()
        } catch (_: CharacterCodingException) {
            throw OpException("utf8", "invalid UTF-8")
        } finally {
            buffer.limit(limit)
        }
    }
}
