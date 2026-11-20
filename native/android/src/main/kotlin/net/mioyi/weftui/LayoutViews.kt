package net.mioyi.weftui

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.MeasureScope
import androidx.compose.ui.layout.ParentDataModifier
import androidx.compose.ui.layout.Placeable
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Density
import kotlin.math.roundToInt

private data class WeftParentData(
    val sizing: WeftSizing? = null,
    val padding: WeftEdges = WeftEdges(),
    val child: WeftChild = WeftChild(),
    val explicitGrow: Boolean = false,
    val explicitFill: Boolean = false,
)

private val defaultParentData = WeftParentData()

private fun Any?.weftData() = (this as? WeftParentData) ?: defaultParentData

/** Adds content-box size constraints for the parent layout to resolve. */
fun Modifier.weftSize(
    width: Float? = null,
    height: Float? = null,
    minWidth: Float? = null,
    maxWidth: Float? = null,
    minHeight: Float? = null,
    maxHeight: Float? = null,
    aspectRatio: Float? = null,
): Modifier = then(WeftSizeModifier(width, height, minWidth, maxWidth, minHeight, maxHeight, aspectRatio))

private data class WeftSizeModifier(
    val width: Float?,
    val height: Float?,
    val minWidth: Float?,
    val maxWidth: Float?,
    val minHeight: Float?,
    val maxHeight: Float?,
    val aspectRatio: Float?,
) : ParentDataModifier {
    override fun Density.modifyParentData(parentData: Any?): Any {
        val data = parentData.weftData()
        return data.copy(
            sizing = WeftSizing(
                width?.times(density)?.toDouble(), height?.times(density)?.toDouble(),
                minWidth?.times(density)?.toDouble(), maxWidth?.times(density)?.toDouble(),
                minHeight?.times(density)?.toDouble(), maxHeight?.times(density)?.toDouble(), aspectRatio?.toDouble(),
            ),
            child = data.child.copy(fixedWidth = width != null, fixedHeight = height != null),
        )
    }
}

/** Adds sizing and placement information belonging to the parent layout. */
fun Modifier.weftChild(
    grow: Float? = null,
    fill: Boolean? = null,
    margin: Float? = null,
    marginX: Float? = null,
    marginY: Float? = null,
    marginTop: Float? = null,
    marginBottom: Float? = null,
    marginStart: Float? = null,
    marginEnd: Float? = null,
    spacer: Boolean = false,
    size: Float? = null,
): Modifier = then(WeftChildModifier(
    grow, fill, edges(margin, marginX, marginY, marginTop, marginBottom, marginStart, marginEnd), spacer, size,
))

private data class WeftChildModifier(
    val grow: Float?,
    val fill: Boolean?,
    val margin: WeftEdges,
    val spacer: Boolean = false,
    val size: Float? = null,
) : ParentDataModifier {
    override fun Density.modifyParentData(parentData: Any?): Any {
        val data = parentData.weftData()
        return data.copy(
            child = data.child.copy(
                grow = grow?.toDouble() ?: data.child.grow, fill = fill ?: data.child.fill,
                margin = margin.pixels(density), spacer = spacer,
                size = size?.let { (it.coerceAtLeast(0f) * density).roundToInt().toDouble() },
            ),
            explicitGrow = grow != null, explicitFill = fill != null,
        )
    }
}

internal fun Modifier.weftPadding(padding: WeftEdges): Modifier =
    if (padding == WeftEdges()) this else then(WeftPaddingModifier(padding))

private data class WeftPaddingModifier(val padding: WeftEdges) : ParentDataModifier {
    override fun Density.modifyParentData(parentData: Any?): Any = parentData.weftData().copy(padding = padding.pixels(density))
}

private object WeftScrollModifier : ParentDataModifier {
    override fun Density.modifyParentData(parentData: Any?): Any {
        val data = parentData.weftData()
        return data.copy(child = data.child.copy(
            grow = if (data.explicitGrow) data.child.grow else 1.0,
            fill = if (data.explicitFill) data.child.fill else true,
        ))
    }
}

internal fun edges(
    all: Float?, x: Float?, y: Float?, top: Float?, bottom: Float?, start: Float?, end: Float?,
): WeftEdges = WeftEdges(
    (top ?: y ?: all ?: 0f).toDouble(), (end ?: x ?: all ?: 0f).toDouble(),
    (bottom ?: y ?: all ?: 0f).toDouble(), (start ?: x ?: all ?: 0f).toDouble(),
)

internal fun paddingEdges(
    all: Float?, x: Float?, y: Float?, top: Float?, bottom: Float?, start: Float?, end: Float?,
): WeftEdges = edges(all, x, y, top, bottom, start, end).let {
    WeftEdges(it.top.coerceAtLeast(0.0), it.end.coerceAtLeast(0.0), it.bottom.coerceAtLeast(0.0), it.start.coerceAtLeast(0.0))
}

private fun WeftEdges.pixels(density: Float) = WeftEdges(
    (top * density).roundToInt().toDouble(), (end * density).roundToInt().toDouble(),
    (bottom * density).roundToInt().toDouble(), (start * density).roundToInt().toDouble(),
)

/** Lays out children along the given axis; this implements both Column and Row. */
@Composable
fun WeftLinear(
    axis: WeftAxis,
    modifier: Modifier = Modifier,
    gap: Float = 0f,
    align: WeftCrossAlign = WeftCrossAlign.START,
    justify: WeftJustify = WeftJustify.START,
    padding: Float? = null,
    paddingX: Float? = null,
    paddingY: Float? = null,
    paddingTop: Float? = null,
    paddingBottom: Float? = null,
    paddingStart: Float? = null,
    paddingEnd: Float? = null,
    background: WeftColor? = null,
    content: @Composable () -> Unit,
) {
    val insets = paddingEdges(padding, paddingX, paddingY, paddingTop, paddingBottom, paddingStart, paddingEnd)
    val box = modifier.weftPadding(insets)
    LinearContent(axis, gap, align, justify, insets, if (background == null) box else box.background(background.resolve()), content)
}

@Composable
private fun LinearContent(
    axis: WeftAxis,
    gap: Float,
    align: WeftCrossAlign,
    justify: WeftJustify,
    padding: WeftEdges,
    modifier: Modifier,
    content: @Composable () -> Unit,
    flexChildren: Boolean = true,
) {
    Layout(content, modifier) { measurables, constraints ->
        placeWeft(measurables, constraints, flexChildren) { children, measure ->
            weftLinear(
                axis, (gap.coerceAtLeast(0f) * density).roundToInt().toDouble(), align, justify,
                padding.pixels(density), constraints.proposal(), children, pixelShares = true, measure = measure,
            )
        }
    }
}

@Composable
internal fun WeftButtonContent(content: @Composable () -> Unit) = LinearContent(
    WeftAxis.HORIZONTAL, 0f, WeftCrossAlign.CENTER, WeftJustify.CENTER, WeftEdges(), Modifier,
    content, flexChildren = false,
)

/** Overlays children in source order, with the last child on top. */
@Composable
fun WeftStack(
    modifier: Modifier = Modifier,
    alignment: WeftAlignment = WeftAlignment.TOP_START,
    padding: Float? = null,
    paddingX: Float? = null,
    paddingY: Float? = null,
    paddingTop: Float? = null,
    paddingBottom: Float? = null,
    paddingStart: Float? = null,
    paddingEnd: Float? = null,
    content: @Composable () -> Unit,
) {
    val insets = paddingEdges(padding, paddingX, paddingY, paddingTop, paddingBottom, paddingStart, paddingEnd)
    Layout(content, modifier.weftPadding(insets)) { measurables, constraints ->
        placeWeft(measurables, constraints) { children, measure ->
            weftStack(alignment, insets.pixels(density), constraints.proposal(), children, measure)
        }
    }
}

/** A linear layout inside a single-axis scrolling viewport. */
@Composable
fun WeftScroll(
    modifier: Modifier = Modifier,
    direction: WeftAxis = WeftAxis.VERTICAL,
    gap: Float = 0f,
    align: WeftCrossAlign = WeftCrossAlign.START,
    indicators: Boolean = true,
    padding: Float? = null,
    paddingX: Float? = null,
    paddingY: Float? = null,
    paddingTop: Float? = null,
    paddingBottom: Float? = null,
    paddingStart: Float? = null,
    paddingEnd: Float? = null,
    content: @Composable () -> Unit,
) {
    val insets = paddingEdges(padding, paddingX, paddingY, paddingTop, paddingBottom, paddingStart, paddingEnd)
    val state = rememberScrollState()
    BoxWithConstraints(modifier.then(WeftScrollModifier).weftPadding(insets), propagateMinConstraints = true) {
        val horizontal = direction == WeftAxis.HORIZONTAL
        // Without a finite viewport there is nothing to scroll, and Compose requires a finite limit.
        val bounded = if (horizontal) constraints.hasBoundedWidth else constraints.hasBoundedHeight
        val viewport = if (bounded) {
            val bar = if (indicators) Modifier.weftScrollbar(state, direction) else Modifier
            bar.scrolling(state, direction)
        } else Modifier
        LinearContent(direction, gap, align, WeftJustify.START, insets, viewport, content)
    }
}

private fun Modifier.scrolling(state: ScrollState, direction: WeftAxis) = when (direction) {
    WeftAxis.VERTICAL -> verticalScroll(state)
    WeftAxis.HORIZONTAL -> horizontalScroll(state)
}

/** A fixed-size or growing empty region in its parent's main axis. */
@Composable
fun WeftSpacer(size: Float? = null) {
    Spacer(Modifier.weftChild(spacer = true, size = size))
}

private fun Constraints.proposal() = WeftProposal(
    WeftLimit(if (hasBoundedWidth) maxWidth.toDouble() else Double.POSITIVE_INFINITY, minWidth.toDouble(), hasFixedWidth),
    WeftLimit(if (hasBoundedHeight) maxHeight.toDouble() else Double.POSITIVE_INFINITY, minHeight.toDouble(), hasFixedHeight),
)

private fun WeftProposal.constraints() = Constraints.fitPrioritizingWidth(
    width.minimum.roundToInt(), if (width.bounded) width.maximum.roundToInt() else Constraints.Infinity,
    height.minimum.roundToInt(), if (height.bounded) height.maximum.roundToInt() else Constraints.Infinity,
)

private inline fun MeasureScope.placeWeft(
    measurables: List<Measurable>,
    bounds: Constraints,
    flexChildren: Boolean = true,
    calculate: (List<WeftChild>, (Int, WeftProposal) -> WeftExtent) -> WeftPlacement,
): androidx.compose.ui.layout.MeasureResult {
    val placeables = arrayOfNulls<Placeable>(measurables.size)
    val children = measurables.map {
        val child = it.parentData.weftData().child
        if (flexChildren || (child.grow == 0.0 && !child.fill)) child else child.copy(grow = 0.0, fill = false)
    }
    val result = calculate(children) { index, proposal ->
        val measurable = measurables[index]
        val data = measurable.parentData.weftData()
        val constrained = data.sizing?.resolve(proposal, data.padding) ?: proposal
        val placeable = measurable.measure(constrained.constraints())
        placeables[index] = placeable
        WeftExtent(placeable.width.toDouble(), placeable.height.toDouble())
    }
    return layout(
        result.size.width.roundToInt().coerceIn(bounds.minWidth, bounds.maxWidth),
        result.size.height.roundToInt().coerceIn(bounds.minHeight, bounds.maxHeight),
    ) {
        result.frames.forEachIndexed { index, frame ->
            checkNotNull(placeables[index]).place(frame.x.roundToInt(), frame.y.roundToInt())
        }
    }
}
