package net.mioyi.weftui

import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL

/**
 * `Image`: a remote image, downloaded by the shell and kept in a memory
 * cache. With one fixed side the other follows the aspect ratio, with both
 * the image fills them, and with neither it keeps its own size, one pixel
 * per dp. Images that fail to load leave the fixed space empty.
 */
@Composable
fun WeftImage(src: String, modifier: Modifier = Modifier) {
    val bitmap by produceState(Images.cached(src), src) {
        if (value == null) value = withContext(Dispatchers.IO) { Images.load(src) }
    }
    val image = bitmap
    if (image == null) {
        Box(modifier.size(0.dp))
        return
    }
    Image(
        image,
        contentDescription = null,
        modifier = modifier.layout { measurable, constraints ->
            val w = image.width.dp.toPx()
            val h = image.height.dp.toPx()
            val scale = when {
                constraints.hasFixedWidth -> constraints.maxWidth / w
                constraints.hasFixedHeight -> constraints.maxHeight / h
                else -> minOf(1f, constraints.maxWidth / w, constraints.maxHeight / h)
            }
            val width = (w * scale).toInt().coerceIn(constraints.minWidth, constraints.maxWidth)
            val height = (h * scale).toInt().coerceIn(constraints.minHeight, constraints.maxHeight)
            val placeable = measurable.measure(Constraints.fixed(width, height))
            layout(width, height) { placeable.place(0, 0) }
        },
        contentScale = ContentScale.FillBounds,
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
            val bytes = connection.inputStream.use(InputStream::readBytes)
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size)?.asImageBitmap()?.also { cache.put(src, it) }
        } catch (error: Exception) {
            WeftLog.warn("weft: cannot load the image $src ($error)")
            null
        } finally {
            connection.disconnect()
        }
    }
}
