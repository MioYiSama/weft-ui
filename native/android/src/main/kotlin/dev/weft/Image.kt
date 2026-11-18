package dev.weft

import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
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
fun WeftImage(src: String, width: Float? = null, height: Float? = null) {
    val bitmap by produceState(Images.cached(src), src) {
        if (value == null) value = withContext(Dispatchers.IO) { Images.load(src) }
    }
    val image = bitmap
    if (image == null) {
        Box(Modifier.size((width ?: 0f).dp, (height ?: 0f).dp))
        return
    }
    val ratio = image.width.toFloat() / image.height.coerceAtLeast(1)
    val modifier = when {
        width != null && height != null -> Modifier.size(width.dp, height.dp)
        width != null -> Modifier.width(width.dp).aspectRatio(ratio)
        height != null -> Modifier.height(height.dp).aspectRatio(ratio)
        else -> Modifier.size(image.width.dp, image.height.dp)
    }
    Image(
        image,
        contentDescription = null,
        modifier = modifier,
        contentScale = if (width != null && height != null) ContentScale.FillBounds else ContentScale.Fit,
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
