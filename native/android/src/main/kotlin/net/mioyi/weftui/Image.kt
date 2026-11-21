package net.mioyi.weftui

import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL
import kotlin.math.roundToInt

/** Displays a density-qualified drawable or a remote image; missing or pending images have no intrinsic size. */
@Composable
fun WeftImage(
    src: String,
    modifier: Modifier = Modifier,
    fit: WeftImageFit = WeftImageFit.COVER,
    placeholder: WeftColor? = null,
) {
    if (src.startsWith("asset:")) {
        val resources = LocalResources.current
        val packageName = LocalContext.current.packageName
        val id = remember(resources, packageName, src) {
            val drawable = resources.weftResource(packageName, "drawable", src.removePrefix("asset:"))
            if (drawable == 0 && WeftLog.debuggable) WeftLog.warn("weft: image asset not found: $src")
            drawable
        }
        if (id != 0) {
            val painter = painterResource(id)
            RenderImage(painter, painter.intrinsicSize, fit, modifier)
            return
        }
    } else {
        val bitmap = remember(src) { mutableStateOf(Images.cached(src)) }
        val image = bitmap.value
        if (image != null) {
            val painter = remember(image) { BitmapPainter(image) }
            val intrinsic = with(LocalDensity.current) { Size(image.width.dp.toPx(), image.height.dp.toPx()) }
            RenderImage(painter, intrinsic, fit, modifier)
            return
        }
        LaunchedEffect(src) { bitmap.value = withContext(Dispatchers.IO) { Images.load(src) } }
    }
    val paint = if (placeholder == null) modifier else modifier.background(placeholder.resolve())
    Box(paint.size(0.dp))
}

@Composable
private fun RenderImage(painter: Painter, intrinsic: Size, fit: WeftImageFit, modifier: Modifier) {
    Image(
        painter,
        contentDescription = null,
        modifier = modifier.layout { measurable, constraints ->
            val (w, h) = intrinsic
            val scale = when {
                constraints.hasFixedWidth -> constraints.maxWidth / w
                constraints.hasFixedHeight -> constraints.maxHeight / h
                else -> minOf(1f, constraints.maxWidth / w, constraints.maxHeight / h)
            }
            val width = (w * scale).roundToInt().coerceIn(constraints.minWidth, constraints.maxWidth)
            val height = (h * scale).roundToInt().coerceIn(constraints.minHeight, constraints.maxHeight)
            val placeable = measurable.measure(Constraints.fixed(width, height))
            layout(width, height) { placeable.place(0, 0) }
        },
        contentScale = when (fit) {
            WeftImageFit.COVER -> ContentScale.Crop
            WeftImageFit.CONTAIN -> ContentScale.Fit
            WeftImageFit.FILL -> ContentScale.FillBounds
        },
    )
}

/** Downloaded images by URL. */
private object Images {
    /** Up to an eighth of the heap, in bytes. */
    private val cache = object : LruCache<String, ImageBitmap>((Runtime.getRuntime().maxMemory() / 8).toInt()) {
        override fun sizeOf(key: String, value: ImageBitmap): Int = value.width * value.height * 4
    }

    fun cached(src: String): ImageBitmap? = cache.get(src)

    /** Downloads and decodes [src]; null when it fails. Blocks. */
    fun load(src: String): ImageBitmap? {
        val connection = try {
            URL(src).openConnection() as? HttpURLConnection ?: return null
        } catch (error: Exception) {
            WeftLog.warn("weft: cannot load the image $src ($error)")
            return null
        }
        return try {
            if (connection.responseCode >= 400) {
                WeftLog.warn("weft: cannot load the image $src (status ${connection.responseCode})")
                return null
            }
            val bitmap = connection.inputStream.use { BitmapFactory.decodeStream(it) }
            bitmap?.asImageBitmap()?.also { cache.put(src, it) }
        } catch (error: Exception) {
            WeftLog.warn("weft: cannot load the image $src ($error)")
            null
        } finally {
            connection.disconnect()
        }
    }
}
