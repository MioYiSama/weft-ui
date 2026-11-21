package net.mioyi.weftui

import org.junit.Assert.assertEquals
import org.junit.Test

class ControlsTest {
    @Test
    fun editLimitsCountUtf16CodeUnitsWithoutSplittingPairs() {
        val text = "a😀b🚀c"
        assertEquals(text, text.limitUtf16(-1))
        assertEquals(text, text.limitUtf16(7))
        assertEquals("", text.limitUtf16(0))
        assertEquals("a", text.limitUtf16(1))
        assertEquals("a", text.limitUtf16(2))
        assertEquals("a😀", text.limitUtf16(3))
        assertEquals("a😀b", text.limitUtf16(5))
        assertEquals("a😀b🚀", text.limitUtf16(6))
        assertEquals("", "😀".limitUtf16(1))
        assertEquals("abc", "abcd".limitUtf16(3))
    }

    @Test
    fun sliderRoundsRelativeToTheMinimumAndBreaksTiesUpward() {
        assertEquals(1f, snapSliderValue(2.9f, min = -7f, max = 13f, step = 4f), 0f)
        assertEquals(5f, snapSliderValue(3f, min = -7f, max = 13f, step = 4f), 0f)
    }

    @Test
    fun sliderClampsToInRangeStepsEvenIfTheMaximumIsNotOnTheGrid() {
        assertEquals(2f, snapSliderValue(0f, min = 2f, max = 12f, step = 4f), 0f)
        assertEquals(10f, snapSliderValue(12f, min = 2f, max = 12f, step = 4f), 0f)
        assertEquals(10f, snapSliderValue(90f, min = 2f, max = 12f, step = 4f), 0f)
    }
}
