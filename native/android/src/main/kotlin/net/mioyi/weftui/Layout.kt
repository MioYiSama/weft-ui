package net.mioyi.weftui

import kotlin.math.round

/** The platform-independent measure and placement rules, in physical units chosen by the caller. */
internal data class WeftLimit(
    val maximum: Double = Double.POSITIVE_INFINITY,
    val minimum: Double = 0.0,
    val exact: Boolean = false,
) {
    val bounded get() = maximum.isFinite()

    fun inset(length: Double) = WeftLimit(
        (maximum - length).coerceAtLeast(0.0), (minimum - length).coerceAtLeast(0.0), exact,
    )
    fun resolve(natural: Double): Double = natural.coerceIn(minimum, maximum)

    companion object {
        fun exact(length: Double) = WeftLimit(length, length, exact = true)
    }
}

internal data class WeftProposal(val width: WeftLimit = WeftLimit(), val height: WeftLimit = WeftLimit())
internal data class WeftExtent(val width: Double, val height: Double)
internal data class WeftFrame(val x: Double, val y: Double, val width: Double, val height: Double)
internal data class WeftPlacement(val size: WeftExtent, val frames: List<WeftFrame>)

internal data class WeftEdges(val top: Double = 0.0, val end: Double = 0.0, val bottom: Double = 0.0, val start: Double = 0.0) {
    val horizontal get() = start + end
    val vertical get() = top + bottom
}

internal data class WeftChild(
    val grow: Double = 0.0,
    val fill: Boolean = false,
    val margin: WeftEdges = WeftEdges(),
    val fixedWidth: Boolean = false,
    val fixedHeight: Boolean = false,
    val spacer: Boolean = false,
    val size: Double? = null,
) {
    val weight get() = if (spacer) (if (size == null) 1.0 else 0.0) else grow
}

internal fun weftLinear(
    axis: WeftAxis,
    gap: Double,
    align: WeftCrossAlign,
    justify: WeftJustify,
    padding: WeftEdges,
    proposal: WeftProposal,
    children: List<WeftChild>,
    pixelShares: Boolean = false,
    measure: (Int, WeftProposal) -> WeftExtent,
): WeftPlacement {
    val horizontal = axis == WeftAxis.HORIZONTAL
    val before = if (horizontal) padding.start else padding.top
    val crossBefore = if (horizontal) padding.top else padding.start
    val mainPadding = if (horizontal) padding.horizontal else padding.vertical
    val crossPadding = if (horizontal) padding.vertical else padding.horizontal
    val main = (if (horizontal) proposal.width else proposal.height).inset(mainPadding)
    val cross = (if (horizontal) proposal.height else proposal.width).inset(crossPadding)
    val count = children.size
    val extents = arrayOfNulls<WeftExtent>(count)
    fun WeftChild.mainMargins() = if (horizontal) margin.horizontal else margin.vertical
    fun WeftChild.crossMargins() = if (horizontal) margin.vertical else margin.horizontal
    fun WeftChild.stretches(): Boolean = !spacer && cross.bounded && (fill || align == WeftCrossAlign.STRETCH) &&
        !(if (horizontal) fixedHeight else fixedWidth)
    fun WeftExtent.main() = if (horizontal) width else height
    fun WeftExtent.cross() = if (horizontal) height else width
    fun measureChild(index: Int, child: WeftChild, mainLimit: WeftLimit): WeftExtent {
        val crossMaximum = (cross.maximum - child.crossMargins()).coerceAtLeast(0.0)
        val crossLimit = if (child.stretches()) WeftLimit.exact(crossMaximum) else WeftLimit(crossMaximum)
        // A spacer still needs to be measured and placed by the host, but never stretches.
        val childMain = if (child.spacer) WeftLimit.exact(child.size ?: mainLimit.minimum) else mainLimit
        val childCross = if (child.spacer) WeftLimit.exact(0.0) else crossLimit
        return measure(index, if (horizontal) WeftProposal(childMain, childCross) else WeftProposal(childCross, childMain))
    }

    var used = gap * (count - 1).coerceAtLeast(0)
    var naturalCross = 0.0
    var totalWeight = 0.0
    var lastGrowing = -1
    for ((index, child) in children.withIndex()) {
        used += child.mainMargins()
        if (child.weight > 0.0 && main.bounded) {
            totalWeight += child.weight
            lastGrowing = index
        } else {
            val measured = measureChild(index, child, WeftLimit())
            extents[index] = measured
            used += measured.main()
            naturalCross = maxOf(naturalCross, measured.cross() + child.crossMargins())
        }
    }
    if (lastGrowing >= 0) {
        val free = (main.maximum - used).coerceAtLeast(0.0)
        var distributed = 0.0
        var weight = 0.0
        for (index in 0..lastGrowing) {
            val child = children[index]
            if (child.weight <= 0.0) continue
            weight += child.weight
            val end = if (index == lastGrowing) free else (free * (weight / totalWeight)).let {
                if (pixelShares) round(it).coerceAtMost(free) else it
            }
            val measured = measureChild(index, child, WeftLimit.exact(end - distributed))
            distributed = end
            extents[index] = measured
            used += measured.main()
            naturalCross = maxOf(naturalCross, measured.cross() + child.crossMargins())
        }
    }

    val mainSize = main.resolve(used)
    val crossSize = cross.resolve(naturalCross)
    val remaining = mainSize - used
    val spacing = when (justify) {
        WeftJustify.SPACE_BETWEEN -> if (count > 1) remaining / (count - 1) else 0.0
        WeftJustify.SPACE_AROUND -> if (count > 0) remaining / count else 0.0
        WeftJustify.SPACE_EVENLY -> remaining / (count + 1)
        else -> 0.0
    }
    var offset = before + when (justify) {
        WeftJustify.CENTER -> remaining / 2
        WeftJustify.END -> remaining
        WeftJustify.SPACE_AROUND -> spacing / 2
        WeftJustify.SPACE_EVENLY -> spacing
        else -> 0.0
    }
    val frames = List(count) { index ->
        val child = children[index]
        val measured = checkNotNull(extents[index])
        val mainOffset = offset + if (horizontal) child.margin.start else child.margin.top
        val crossRoom = crossSize - measured.cross() - child.crossMargins()
        val crossOffset = crossBefore + (if (horizontal) child.margin.top else child.margin.start) +
            if (child.stretches() || child.fill) 0.0 else when (align) {
                WeftCrossAlign.CENTER -> crossRoom / 2
                WeftCrossAlign.END -> crossRoom
                else -> 0.0
            }
        offset += measured.main() + child.mainMargins() + gap + spacing
        if (horizontal) WeftFrame(mainOffset, crossOffset, measured.width, measured.height)
        else WeftFrame(crossOffset, mainOffset, measured.width, measured.height)
    }
    return WeftPlacement(
        if (horizontal) WeftExtent(mainSize + mainPadding, crossSize + crossPadding)
        else WeftExtent(crossSize + crossPadding, mainSize + mainPadding),
        frames,
    )
}

internal fun weftStack(
    alignment: WeftAlignment,
    padding: WeftEdges,
    proposal: WeftProposal,
    children: List<WeftChild>,
    measure: (Int, WeftProposal) -> WeftExtent,
): WeftPlacement {
    val width = proposal.width.inset(padding.horizontal)
    val height = proposal.height.inset(padding.vertical)
    val extents = arrayOfNulls<WeftExtent>(children.size)
    var naturalWidth = 0.0
    var naturalHeight = 0.0
    for ((index, child) in children.withIndex()) {
        if (child.fill) continue
        val measureProposal = WeftProposal(
            WeftLimit((width.maximum - child.margin.horizontal).coerceAtLeast(0.0)),
            WeftLimit((height.maximum - child.margin.vertical).coerceAtLeast(0.0)),
        )
        val measured = measure(index, measureProposal)
        extents[index] = measured
        naturalWidth = maxOf(naturalWidth, measured.width + child.margin.horizontal)
        naturalHeight = maxOf(naturalHeight, measured.height + child.margin.vertical)
    }
    val contentWidth = width.resolve(naturalWidth)
    val contentHeight = height.resolve(naturalHeight)
    val xFraction = when (alignment) {
        WeftAlignment.TOP_START, WeftAlignment.START, WeftAlignment.BOTTOM_START -> 0.0
        WeftAlignment.TOP, WeftAlignment.CENTER, WeftAlignment.BOTTOM -> 0.5
        WeftAlignment.TOP_END, WeftAlignment.END, WeftAlignment.BOTTOM_END -> 1.0
    }
    val yFraction = when (alignment) {
        WeftAlignment.TOP_START, WeftAlignment.TOP, WeftAlignment.TOP_END -> 0.0
        WeftAlignment.START, WeftAlignment.CENTER, WeftAlignment.END -> 0.5
        WeftAlignment.BOTTOM_START, WeftAlignment.BOTTOM, WeftAlignment.BOTTOM_END -> 1.0
    }
    val frames = List(children.size) { index ->
        val margin = children[index].margin
        val horizontal = contentWidth - margin.horizontal
        val vertical = contentHeight - margin.vertical
        val measured = extents[index] ?: measure(index, WeftProposal(
            WeftLimit.exact(horizontal.coerceAtLeast(0.0)),
            WeftLimit.exact(vertical.coerceAtLeast(0.0)),
        ))
        WeftFrame(
            padding.start + margin.start + (horizontal - measured.width) * xFraction,
            padding.top + margin.top + (vertical - measured.height) * yFraction,
            measured.width,
            measured.height,
        )
    }
    return WeftPlacement(WeftExtent(contentWidth + padding.horizontal, contentHeight + padding.vertical), frames)
}

/** Content-box constraints are applied to an incoming border-box proposal before measurement. */
internal data class WeftSizing(
    val width: Double? = null,
    val height: Double? = null,
    val minWidth: Double? = null,
    val maxWidth: Double? = null,
    val minHeight: Double? = null,
    val maxHeight: Double? = null,
    val aspectRatio: Double? = null,
) {
    fun resolve(proposal: WeftProposal, padding: WeftEdges, border: Double = 0.0): WeftProposal {
        val horizontal = padding.horizontal + 2 * border
        val vertical = padding.vertical + 2 * border
        var w = proposal.width.constrain(width, minWidth, maxWidth, horizontal)
        var h = proposal.height.constrain(height, minHeight, maxHeight, vertical)
        val ratio = aspectRatio
        if (ratio != null && ratio > 0) {
            if (w.exact && height == null) {
                h = proposal.height.constrain((w.maximum - horizontal).coerceAtLeast(0.0) / ratio, minHeight, maxHeight, vertical)
            } else if (h.exact && width == null) {
                w = proposal.width.constrain((h.maximum - vertical).coerceAtLeast(0.0) * ratio, minWidth, maxWidth, horizontal)
            }
        }
        return WeftProposal(w, h)
    }

    private fun WeftLimit.constrain(fixed: Double?, min: Double?, max: Double?, insets: Double): WeftLimit {
        val lower = (min ?: 0.0).coerceAtLeast(0.0) + insets
        val upper = ((max ?: Double.POSITIVE_INFINITY) + insets).coerceAtLeast(lower)
        if (fixed != null) return WeftLimit.exact((fixed.coerceAtLeast(0.0) + insets).coerceIn(lower, upper))
        return WeftLimit(maximum.coerceIn(lower, upper), minimum.coerceIn(lower, upper), exact)
    }
}
