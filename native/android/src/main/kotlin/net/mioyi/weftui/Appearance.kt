package net.mioyi.weftui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.addOutline
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.platform.LocalGraphicsContext
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

private val smallShadow = Shadow(3.dp, Color(0x1f000000), offset = DpOffset(0.dp, 1.dp))
private val mediumShadow = Shadow(12.dp, Color(0x26000000), offset = DpOffset(0.dp, 4.dp))
private val largeShadow = Shadow(32.dp, Color(0x2e000000), offset = DpOffset(0.dp, 12.dp))

private fun WeftShadow.composeShadow(): Shadow? = when (this) {
    WeftShadow.NONE -> null
    WeftShadow.SMALL -> smallShadow
    WeftShadow.MEDIUM -> mediumShadow
    WeftShadow.LARGE -> largeShadow
}

/** Applies visibility and draws the border box outside the element's content and padding. */
@Composable
fun Modifier.weftAppearance(
    opacity: Float = 1f,
    hidden: Boolean = false,
    background: WeftColor? = null,
    radius: Float = 0f,
    border: Float = 0f,
    borderColor: WeftColor? = null,
    shadow: WeftShadow = WeftShadow.NONE,
    clip: Boolean = false,
): Modifier {
    if (hidden) return weftHidden()
    val preset = shadow.composeShadow()
    val alpha = opacity.coerceIn(0f, 1f)
    var box = when {
        alpha == 1f -> this
        alpha == 0f || (clip && preset == null) -> alpha(alpha)
        // Modifier.alpha clips its layer to the element's bounds, losing shadows and overflow.
        else -> {
            val layerAlpha = (alpha * 255).roundToInt()
            drawWithContent {
                drawIntoCanvas { canvas ->
                    val native = canvas.nativeCanvas
                    val saved = native.saveLayerAlpha(null, layerAlpha)
                    try {
                        drawContent()
                    } finally {
                        native.restoreToCount(saved)
                    }
                }
            }
        }
    }
    if (preset == null && !clip && background == null && border <= 0f) return box
    val shape = roundedShape(radius)
    val fill = background?.resolve()
    if (preset != null) {
        box = if (fill?.alpha == 1f) box.dropShadow(shape, preset) else box.outsetShadow(shape, preset)
    }
    if (clip) box = box.clip(shape)
    if (fill != null) box = box.background(fill, shape)
    if (border > 0f) {
        val ink = borderColor?.resolve() ?: if (isSystemInDarkTheme()) Color(0x26ffffff) else Color(0x1f000000)
        box = box.border(border.dp, ink, shape).weftBorder(border).padding(border.dp)
        if (clip) box = box.clip(roundedShape(radius - border))
    }
    return box
}

private fun roundedShape(radius: Float) = if (radius > 0f) RoundedCornerShape(radius.dp) else RectangleShape

// The built-in modifier also paints under the element; that ink must not show through a translucent fill.
@Composable
private fun Modifier.outsetShadow(shape: Shape, shadow: Shadow): Modifier {
    val shadows = LocalGraphicsContext.current.shadowContext
    return drawWithCache {
        val outline = Path().apply { addOutline(shape.createOutline(size, layoutDirection, this@drawWithCache)) }
        val painter = shadows.createDropShadowPainter(shape, shadow)
        onDrawWithContent {
            clipPath(outline, ClipOp.Difference) { with(painter) { draw(size) } }
            drawContent()
        }
    }
}
