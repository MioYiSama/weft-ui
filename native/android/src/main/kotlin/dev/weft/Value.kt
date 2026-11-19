package dev.weft

import androidx.compose.ui.graphics.Color
import java.io.ByteArrayOutputStream

/** A value of a value slot or an event payload (docs/src/content/docs/protocol.md, "Binary encoding"). */
sealed class WeftValue {
    data class Bool(val value: Boolean) : WeftValue()

    data class Number(val value: Double) : WeftValue()

    data class Text(val value: String) : WeftValue()

    /** The value of a `String` slot. */
    val string: String get() = (this as? Text)?.value ?: ""

    /** The value of a `Bool` slot. */
    val bool: Boolean get() = (this as? Bool)?.value ?: false

    /** The value of an `Int` slot: a length in dp. */
    val length: Float get() = (this as? Number)?.value?.toFloat() ?: 0f

    /** The value of a `Color` slot; invalid colors are transparent. */
    val color: Color
        get() = parseColor(string) ?: run {
            if (WeftLog.debuggable) WeftLog.warn("weft: invalid color $string, using transparent")
            Color.Transparent
        }

    /** The binary encoding: a tag byte and the data. */
    internal fun encode(): ByteArray = when (this) {
        is Bool -> byteArrayOf(if (value) 1 else 0)
        is Number -> ByteArray(9).also { bytes ->
            bytes[0] = 2
            val bits = value.toRawBits()
            for (i in 0 until 8) bytes[1 + i] = (bits ushr (8 * i)).toByte()
        }
        is Text -> {
            val utf8 = value.encodeToByteArray()
            ByteArrayOutputStream(5 + utf8.size).apply {
                write(3)
                for (i in 0 until 4) write(utf8.size ushr (8 * i))
                write(utf8)
            }.toByteArray()
        }
    }
}

/** Parses `#RGB`, `#RRGGBB` or `#RRGGBBAA` in either hex case. */
internal fun parseColor(text: String): Color? {
    if (!text.startsWith('#')) return null
    val digits = text.substring(1)
    if (digits.length !in intArrayOf(3, 6, 8) || !digits.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }) {
        return null
    }
    val value = digits.toLong(16)
    val rgba = when (digits.length) {
        3 -> {
            val r = value shr 8 and 0xf
            val g = value shr 4 and 0xf
            val b = value and 0xf
            (r * 17 shl 24) or (g * 17 shl 16) or (b * 17 shl 8) or 0xff
        }
        6 -> value shl 8 or 0xff
        else -> value
    }
    // Compose colors are ARGB.
    return Color(((rgba and 0xff) shl 24) or (rgba ushr 8))
}
