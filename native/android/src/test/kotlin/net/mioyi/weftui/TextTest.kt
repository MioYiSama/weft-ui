package net.mioyi.weftui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.junit.Assert.assertEquals
import org.junit.Test

class TextTest {
    @Test
    fun spansPreserveContentAndStyleOnlyTheirOwnRanges() {
        val text = buildAnnotatedString {
            appendWeftSpan("One ", weight = WeftWeight.SEMIBOLD)
            appendWeftSpan("")
            appendWeftSpan("🌱", color = Color.Green, size = 18.sp, decoration = WeftDecoration.UNDERLINE)
            appendWeftSpan(" two", italic = false, decoration = WeftDecoration.NONE)
            appendWeftSpan("\nend")
        }
        assertEquals("One 🌱 two\nend", text.text)
        assertEquals(
            listOf(
                AnnotatedString.Range(SpanStyle(fontWeight = FontWeight.W600), 0, 4),
                AnnotatedString.Range(
                    SpanStyle(color = Color.Green, fontSize = 18.sp, textDecoration = TextDecoration.Underline), 4, 6,
                ),
                AnnotatedString.Range(SpanStyle(fontStyle = FontStyle.Normal, textDecoration = TextDecoration.None), 6, 10),
            ),
            text.spanStyles,
        )
    }

    @Test
    fun lineHeightAndFontSizeUseTheSameDpToSpConversion() = with(Density(2f, 1.5f)) {
        val base = TextStyle(fontSize = 15.sp, lineHeight = 22.sp)
        assertEquals(base, weftTypography(base))
        val style = weftTypography(base, size = 18f, lineHeight = 27f, align = WeftTextAlign.END)
        assertEquals(18.dp.toSp(), style.fontSize)
        assertEquals(27.dp.toSp(), style.lineHeight)
        assertEquals(LineHeightStyle.Trim.None, style.lineHeightStyle?.trim)
        assertEquals(TextAlign.End, style.textAlign)
    }

    @Test
    fun nonpositiveLineLimitsAreUnlimited() {
        assertEquals(0, null.lineLimit())
        assertEquals(0, 0f.lineLimit())
        assertEquals(0, (-2f).lineLimit())
        assertEquals(2, 2f.lineLimit())
    }

    @Test
    fun allFiveWeightsAreSupported() = with(Density(1f)) {
        val expected = mapOf(
            WeftWeight.LIGHT to FontWeight.W300,
            WeftWeight.REGULAR to FontWeight.W400,
            WeftWeight.MEDIUM to FontWeight.W500,
            WeftWeight.SEMIBOLD to FontWeight.W600,
            WeftWeight.BOLD to FontWeight.W700,
        )
        for ((weight, expectedWeight) in expected) {
            assertEquals(expectedWeight, weftTypography(TextStyle.Default, weight = weight).fontWeight)
        }
    }
}
