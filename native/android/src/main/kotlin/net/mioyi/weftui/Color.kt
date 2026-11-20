package net.mioyi.weftui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

/** A fixed color or a reference resolved against the current root's theme. */
@Immutable
sealed class WeftColor {
    data class Fixed(val color: Color) : WeftColor()

    private data class Token(val name: String) : WeftColor()

    @Composable
    fun resolve(): Color = when (this) {
        is Fixed -> color
        is Token -> LocalWeftTheme.current.resolve(name, isSystemInDarkTheme())
    }

    companion object {
        val Transparent: WeftColor = Fixed(Color.Transparent)

        fun token(name: String): WeftColor = Token(name)

        /** Parses a theme token or `#RGB`, `#RRGGBB`, or `#RRGGBBAA` (either hex case). */
        fun parse(text: String): WeftColor {
            if (text.length > 1 && text[0] == '@') return token(text.substring(1))
            val color = parseFixedColor(text)
            if (color != null) return color
            if (WeftLog.debuggable) WeftLog.warn("weft: invalid color $text, using transparent")
            return Transparent
        }
    }
}

/** The light and dark color pairs registered by a generated app. */
class WeftTheme private constructor(private val tokens: Map<String, Pair<Color, Color>>) {
    internal fun resolve(name: String, dark: Boolean): Color {
        val token = tokens[name]
        if (token != null) return if (dark) token.second else token.first
        if (WeftLog.debuggable) WeftLog.warn("weft: unknown theme token @$name, using transparent")
        return Color.Transparent
    }

    companion object {
        internal val Empty = WeftTheme(emptyMap())

        /** Registers a generated app's color table for its root. The table must not be mutated. */
        fun register(tokens: Map<String, Pair<Color, Color>>): WeftTheme = WeftTheme(tokens)
    }
}

internal val LocalWeftTheme = staticCompositionLocalOf { WeftTheme.Empty }

private fun parseFixedColor(text: String): WeftColor.Fixed? {
    if ((text.length != 4 && text.length != 7 && text.length != 9) || text[0] != '#') return null
    var rgba = 0L
    for (i in 1 until text.length) {
        val ch = text[i]
        val digit = when (ch) {
            in '0'..'9' -> ch - '0'
            in 'a'..'f' -> ch - 'a' + 10
            in 'A'..'F' -> ch - 'A' + 10
            else -> return null
        }
        rgba = (rgba shl 4) or digit.toLong()
    }
    rgba = when (text.length) {
        4 -> {
            val r = rgba shr 8 and 0xf
            val g = rgba shr 4 and 0xf
            val b = rgba and 0xf
            (r * 17 shl 24) or (g * 17 shl 16) or (b * 17 shl 8) or 0xff
        }
        7 -> rgba shl 8 or 0xff
        else -> rgba
    }
    // Compose colors are ARGB.
    return WeftColor.Fixed(Color(((rgba and 0xff) shl 24) or (rgba ushr 8)))
}
