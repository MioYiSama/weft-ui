package net.mioyi.weftui

import java.io.ByteArrayOutputStream

/** A value of a value slot or an event payload (protocol/SPEC.md, "Binary encoding"). */
sealed class WeftValue {
    data class Bool(val value: Boolean) : WeftValue()

    data class Number(val value: Double) : WeftValue()

    data class Text(val value: String) : WeftValue()

    data class Strings(val value: List<String>) : WeftValue()

    /** The value of a `Strings` slot. */
    val strings: List<String> get() = (this as? Strings)?.value ?: emptyList()

    /** The value of a `String` slot. */
    val string: String get() = (this as? Text)?.value ?: ""

    /** The value of a `Bool` slot. */
    val bool: Boolean get() = (this as? Bool)?.value ?: false

    /** The value of an `Int` or `Float` slot (lengths use dp). */
    val number: Float get() = (this as? Number)?.value?.toFloat() ?: 0f

    /** The fixed color or theme token of a `Color` slot. */
    val color: WeftColor get() = WeftColor.parse(string)

    /** The binary encoding of an event payload: a tag byte and the data. */
    internal fun encode(): ByteArray = when (this) {
        is Bool -> byteArrayOf((if (value) ValueTag.TRUE else ValueTag.FALSE).toByte())
        is Number -> ByteArray(9).also { bytes ->
            bytes[0] = ValueTag.NUMBER.toByte()
            val bits = value.toRawBits()
            for (i in 0 until 8) bytes[1 + i] = (bits ushr (8 * i)).toByte()
        }
        is Text -> {
            val utf8 = value.encodeToByteArray()
            ByteArrayOutputStream(5 + utf8.size).apply {
                write(ValueTag.STRING)
                for (i in 0 until 4) write(utf8.size ushr (8 * i))
                write(utf8)
            }.toByteArray()
        }
        is Strings -> error("weft: string lists are not event payloads")
    }
}
