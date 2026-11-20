package net.mioyi.weftui

import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp

/**
 * Starts the app's session on first use and renders its root component, or
 * why the app failed. The page scrolls vertically, its content starts at the
 * top start corner inside the system bars.
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
        Column(Modifier.fillMaxSize().safeDrawingPadding().verticalScroll(rememberScrollState())) {
            val failure = session.failure
            val root = session.root
            if (failure != null) {
                Text(failure, Modifier.padding(16.dp), color = Color.Red)
            } else if (root != null) {
                CompositionLocalProvider(LocalWeftTheme provides theme) {
                    key(root.id) { root.Content() }
                }
            }
        }
    }
}

/** `Column`: children from the top, aligned to the start, [gap] dp apart. */
@Composable
fun WeftColumn(
    gap: Float = 0f,
    padding: Float? = null,
    width: Float? = null,
    height: Float? = null,
    background: WeftColor? = null,
    content: @Composable () -> Unit,
) {
    Column(
        Modifier.weftBox(padding, width, height, background?.resolve()),
        verticalArrangement = Arrangement.spacedBy(gap.dp),
        horizontalAlignment = Alignment.Start,
    ) { content() }
}

/** `Row`: children from the start, aligned to the top, [gap] dp apart. */
@Composable
fun WeftRow(
    gap: Float = 0f,
    padding: Float? = null,
    width: Float? = null,
    height: Float? = null,
    background: WeftColor? = null,
    content: @Composable () -> Unit,
) {
    Row(
        Modifier.weftBox(padding, width, height, background?.resolve()),
        horizontalArrangement = Arrangement.spacedBy(gap.dp),
        verticalAlignment = Alignment.Top,
    ) { content() }
}

/**
 * Layout properties of `Column` and `Row`: the background behind the
 * padding, the padding around a fixed content size, as CSS boxes do.
 */
private fun Modifier.weftBox(padding: Float?, width: Float?, height: Float?, background: Color?): Modifier {
    var modifier = this
    if (background != null) modifier = modifier.background(background)
    if (padding != null) modifier = modifier.padding(padding.dp)
    if (width != null) modifier = modifier.width(width.dp)
    if (height != null) modifier = modifier.height(height.dp)
    return modifier
}

/** `Text`; unset style properties keep the theme's. Sizes are dp, like every length. */
@Composable
fun WeftText(value: String, color: WeftColor? = null, size: Float? = null, bold: Boolean? = null) {
    val fontSize = if (size != null) with(LocalDensity.current) { size.dp.toSp() } else TextUnit.Unspecified
    Text(
        value,
        color = color?.resolve() ?: Color.Unspecified,
        fontSize = fontSize,
        fontWeight = if (bold == true) FontWeight.Bold else null,
    )
}

/** `Button`: a Material button around its children. */
@Composable
fun WeftButton(onClick: () -> Unit, disabled: Boolean = false, content: @Composable () -> Unit) {
    Button(onClick = onClick, enabled = !disabled) { content() }
}
