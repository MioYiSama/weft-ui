package net.mioyi.weftui

import androidx.compose.foundation.layout.absolutePadding
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp

/** `Text`: renders a plain value or a single paragraph of spans. Sizes are dp, like every length. */
@Composable
fun WeftText(
    value: String = "",
    color: WeftColor? = null,
    size: Float? = null,
    bold: Boolean? = null,
    weight: WeftWeight? = null,
    italic: Boolean? = null,
    decoration: WeftDecoration? = null,
    align: WeftTextAlign? = null,
    maxLines: Float? = null,
    lineHeight: Float? = null,
    modifier: Modifier = Modifier,
    padding: Float? = null,
    paddingX: Float? = null,
    paddingY: Float? = null,
    paddingTop: Float? = null,
    paddingBottom: Float? = null,
    paddingStart: Float? = null,
    paddingEnd: Float? = null,
    spans: (@Composable AnnotatedString.Builder.() -> Unit)? = null,
) {
    val insets = paddingEdges(padding, paddingX, paddingY, paddingTop, paddingBottom, paddingStart, paddingEnd)
    val box = if (insets == WeftEdges()) modifier else modifier.weftPadding(insets).absolutePadding(
        left = insets.start.dp, top = insets.top.dp, right = insets.end.dp, bottom = insets.bottom.dp,
    )
    val textColor = color?.resolve() ?: Color.Unspecified
    val style = with(LocalDensity.current) {
        weftTypography(LocalTextStyle.current, size, bold, weight, italic, decoration, align, lineHeight)
    }
    val limit = maxLines.lineLimit()
    val lines = if (limit > 0) limit else Int.MAX_VALUE
    val overflow = if (limit > 0) TextOverflow.Ellipsis else TextOverflow.Clip
    if (spans == null) {
        Text(value, box, textColor, overflow = overflow, maxLines = lines, style = style)
    } else {
        Text(buildAnnotatedString { spans() }, box, textColor, overflow = overflow, maxLines = lines, style = style)
    }
}

/** `Span`: appends a styled run to the parent `Text` without emitting a view. */
@Composable
fun AnnotatedString.Builder.WeftSpan(
    value: String,
    color: WeftColor? = null,
    size: Float? = null,
    weight: WeftWeight? = null,
    italic: Boolean? = null,
    decoration: WeftDecoration? = null,
) {
    appendWeftSpan(
        value, color?.resolve() ?: Color.Unspecified, with(LocalDensity.current) { textSize(size) },
        weight, italic, decoration,
    )
}

internal fun AnnotatedString.Builder.appendWeftSpan(
    value: String,
    color: Color = Color.Unspecified,
    size: TextUnit = TextUnit.Unspecified,
    weight: WeftWeight? = null,
    italic: Boolean? = null,
    decoration: WeftDecoration? = null,
) {
    if (value.isEmpty()) return
    val start = length
    append(value)
    if (color != Color.Unspecified || size != TextUnit.Unspecified || weight != null || italic != null || decoration != null) {
        addStyle(
            SpanStyle(
                color = color,
                fontSize = size,
                fontWeight = weight?.compose(),
                fontStyle = italic.fontStyle(),
                textDecoration = decoration?.compose(),
            ),
            start, length,
        )
    }
}

internal fun Density.weftTypography(
    base: TextStyle,
    size: Float? = null,
    bold: Boolean? = null,
    weight: WeftWeight? = null,
    italic: Boolean? = null,
    decoration: WeftDecoration? = null,
    align: WeftTextAlign? = null,
    lineHeight: Float? = null,
): TextStyle = base.merge(
    fontSize = textSize(size),
    // The legacy property follows weight in the schema, so it takes precedence when specified.
    fontWeight = bold?.let { if (it) FontWeight.W700 else FontWeight.W400 } ?: weight?.compose(),
    fontStyle = italic.fontStyle(),
    textDecoration = decoration?.compose(),
    textAlign = when (align) {
        WeftTextAlign.START -> TextAlign.Start
        WeftTextAlign.CENTER -> TextAlign.Center
        WeftTextAlign.END -> TextAlign.End
        null -> TextAlign.Unspecified
    },
    lineHeight = textSize(lineHeight),
    lineHeightStyle = if (lineHeight != null && lineHeight >= 0f) explicitLineHeightStyle else null,
)

private val explicitLineHeightStyle = LineHeightStyle(LineHeightStyle.Alignment.Center, LineHeightStyle.Trim.None)

internal fun Float?.lineLimit(): Int = if (this != null && this > 0f) toInt() else 0

private fun Density.textSize(size: Float?): TextUnit =
    if (size != null && size >= 0f) size.dp.toSp() else TextUnit.Unspecified

private fun WeftWeight.compose(): FontWeight = when (this) {
    WeftWeight.LIGHT -> FontWeight.W300
    WeftWeight.REGULAR -> FontWeight.W400
    WeftWeight.MEDIUM -> FontWeight.W500
    WeftWeight.SEMIBOLD -> FontWeight.W600
    WeftWeight.BOLD -> FontWeight.W700
}

private fun Boolean?.fontStyle(): FontStyle? = when (this) {
    true -> FontStyle.Italic
    false -> FontStyle.Normal
    null -> null
}

private fun WeftDecoration.compose(): TextDecoration = when (this) {
    WeftDecoration.NONE -> TextDecoration.None
    WeftDecoration.UNDERLINE -> TextDecoration.Underline
    WeftDecoration.LINE_THROUGH -> TextDecoration.LineThrough
}
