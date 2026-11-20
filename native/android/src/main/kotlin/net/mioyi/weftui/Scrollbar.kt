package net.mioyi.weftui

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest

@Composable
internal fun Modifier.weftScrollbar(state: ScrollState, axis: WeftAxis): Modifier {
    val alpha = remember(state) { Animatable(0f) }
    val color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f)
    LaunchedEffect(state) {
        snapshotFlow { state.isScrollInProgress }.collectLatest { scrolling ->
            if (scrolling) {
                alpha.snapTo(1f)
            } else if (alpha.value != 0f) {
                delay(1_000)
                alpha.animateTo(0f, tween(durationMillis = 200))
            }
        }
    }
    return drawWithContent {
        drawContent()
        val opacity = alpha.value
        val range = state.maxValue
        if (opacity == 0f || range == 0 || range == Int.MAX_VALUE) return@drawWithContent

        val horizontal = axis == WeftAxis.HORIZONTAL
        val length = if (horizontal) size.width else size.height
        val thickness = 3.dp.toPx()
        val thumb = length * (length / (length + range))
        val offset = (length - thumb) * (state.value.toFloat() / range)
        drawRoundRect(
            color = color,
            topLeft = if (horizontal) Offset(offset, size.height - thickness) else Offset(size.width - thickness, offset),
            size = if (horizontal) Size(thumb, thickness) else Size(thickness, thumb),
            cornerRadius = CornerRadius(thickness / 2),
            alpha = opacity,
        )
    }
}
