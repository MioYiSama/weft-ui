package net.mioyi.weftui

import android.content.res.Configuration
import android.content.res.Resources
import org.json.JSONObject

internal data class WeftEnvironment(
    val dark: Boolean,
    val width: Float,
    val height: Float,
    val scale: Float,
    val top: Float = 0f,
    val bottom: Float = 0f,
    val start: Float = 0f,
    val end: Float = 0f,
) {
    fun json(): String = JSONObject()
        .put("colorScheme", if (dark) "dark" else "light")
        .put("screen", JSONObject().put("width", width.toDouble()).put("height", height.toDouble())
            .put("scale", scale.toDouble()))
        .put("safeArea", JSONObject().put("top", top.toDouble()).put("bottom", bottom.toDouble())
            .put("start", start.toDouble()).put("end", end.toDouble()))
        .put("orientation", if (width > height) "landscape" else "portrait")
        .toString()

    companion object {
        fun initial(resources: Resources): WeftEnvironment {
            val config = resources.configuration
            val metrics = resources.displayMetrics
            return WeftEnvironment(
                dark = config.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES,
                width = metrics.widthPixels / metrics.density,
                height = metrics.heightPixels / metrics.density,
                scale = metrics.density,
            )
        }
    }
}
