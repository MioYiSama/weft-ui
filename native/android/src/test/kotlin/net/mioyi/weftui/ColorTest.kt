package net.mioyi.weftui

import androidx.compose.ui.graphics.Color
import org.junit.Assert.assertEquals
import org.junit.Test

class ColorTest {
    @Test
    fun parsesFixedColorsWithCssAlphaOrdering() {
        assertEquals(WeftColor.Fixed(Color(0xffaa00ffL)), WeftColor.parse("#A0f"))
        assertEquals(WeftColor.Fixed(Color(0xff102030L)), WeftColor.parse("#102030"))
        assertEquals(WeftColor.Fixed(Color(0x40102030L)), WeftColor.parse("#10203040"))
    }

    @Test
    fun colorSlotsKeepTokensAndRejectInvalidColors() {
        assertEquals(WeftColor.token("accent"), WeftValue.Text("@accent").color)
        assertEquals(WeftColor.Fixed(Color(0xff102030L)), WeftValue.Text("#102030").color)
        for (text in listOf("", "@", "#12", "#12345g", "#+12", "#１２３")) {
            assertEquals(text, WeftColor.Fixed(Color.Transparent), WeftValue.Text(text).color)
        }
    }

    @Test
    fun aRegisteredThemeResolvesEitherAppearanceAndKeepsTokensRootLocal() {
        val light = Color(0xff0a84ffL)
        val dark = Color(0xff409cffL)
        val theme = WeftTheme.register(mapOf("accent" to (light to dark), "fixed" to (light to light)))
        assertEquals(light, theme.resolve("accent", dark = false))
        assertEquals(dark, theme.resolve("accent", dark = true))
        assertEquals(light, theme.resolve("fixed", dark = true))
        assertEquals(Color.Transparent, theme.resolve("missing", dark = false))
        assertEquals(Color.Transparent, WeftTheme.Empty.resolve("accent", dark = true))
    }
}
