package net.mioyi.weftui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.absolutePadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp

/**
 * Starts the app's session and renders its root component or startup failure inside the system bars.
 */
@Composable
fun WeftRoot(layoutHash: ULong, theme: WeftTheme = WeftTheme.Empty, make: WeftMake) {
    val context = LocalContext.current
    val session = remember { WeftSession.start(context, layoutHash.toLong(), make) }
    DisposableEffect(session) { onDispose { session.close() } }
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val safe = WindowInsets.safeDrawing
        val environment = WeftEnvironment(
            dark = isSystemInDarkTheme(),
            width = maxWidth.value,
            height = maxHeight.value,
            scale = density.density,
            top = with(density) { safe.getTop(this).toDp().value },
            bottom = with(density) { safe.getBottom(this).toDp().value },
            start = with(density) { safe.getLeft(this, LayoutDirection.Ltr).toDp().value },
            end = with(density) { safe.getRight(this, LayoutDirection.Ltr).toDp().value },
        )
        LaunchedEffect(session, environment) { session.updateEnvironment(environment) }
        WeftLinear(
            WeftAxis.VERTICAL,
            Modifier.fillMaxSize().safeDrawingPadding().clipToBounds(),
        ) {
            val failure = session.failure
            val root = session.root
            if (failure != null) {
                Text(failure, Modifier.padding(16.dp), color = Color.Red)
            } else if (root != null) {
                CompositionLocalProvider(LocalWeftTheme provides theme, LocalLayoutDirection provides LayoutDirection.Ltr) {
                    key(root.id) { root.Content() }
                }
            }
        }
    }
}

/** `Text`; unset style properties keep the theme's. Sizes are dp, like every length. */
@Composable
fun WeftText(
    value: String,
    color: WeftColor? = null,
    size: Float? = null,
    bold: Boolean? = null,
    modifier: Modifier = Modifier,
    padding: Float? = null,
    paddingX: Float? = null,
    paddingY: Float? = null,
    paddingTop: Float? = null,
    paddingBottom: Float? = null,
    paddingStart: Float? = null,
    paddingEnd: Float? = null,
) {
    val insets = paddingEdges(padding, paddingX, paddingY, paddingTop, paddingBottom, paddingStart, paddingEnd)
    val box = if (insets == WeftEdges()) modifier else modifier.weftPadding(insets).absolutePadding(
        left = insets.start.dp, top = insets.top.dp, right = insets.end.dp, bottom = insets.bottom.dp,
    )
    val fontSize = if (size != null) with(LocalDensity.current) { size.dp.toSp() } else TextUnit.Unspecified
    Text(
        value,
        modifier = box,
        color = color?.resolve() ?: Color.Unspecified,
        fontSize = fontSize,
        fontWeight = if (bold == true) FontWeight.Bold else null,
    )
}

/** `Button`: a Material button around its children. */
@Composable
fun WeftButton(
    onClick: () -> Unit,
    disabled: Boolean = false,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Button(onClick = onClick, enabled = !disabled, modifier = modifier) {
        WeftButtonContent(content)
    }
}
