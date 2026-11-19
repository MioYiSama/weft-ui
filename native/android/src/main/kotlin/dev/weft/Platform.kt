package dev.weft

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

/**
 * The platform modules of the shell; byte formats in
 * docs/src/content/docs/protocol.md ("Platform calls"). Calls run on a
 * background thread.
 */
internal class PlatformModules(context: Context) {
    private val preferences = context.getSharedPreferences(STORAGE, Context.MODE_PRIVATE)

    /** Starts a call; [done] runs on a background thread with the result or the error message. */
    fun call(module: String, method: String, args: ByteArray, done: (ok: Boolean, result: ByteArray) -> Unit) {
        background.execute {
            val result = try {
                Result.success(call(module, method, args))
            } catch (error: Exception) {
                Result.failure(error)
            }
            result.fold(
                onSuccess = { done(true, it) },
                onFailure = { done(false, (it.message ?: it.toString()).encodeToByteArray()) },
            )
        }
    }

    private fun call(module: String, method: String, args: ByteArray): ByteArray = when (module to method) {
        "fetch" to "request" -> fetch(args)
        "storage" to "get", "storage" to "set", "storage" to "remove" -> storage(method, args)
        else -> throw PlatformFailure("weft: $module.$method is not implemented")
    }

    /** `fetch.request` through HttpURLConnection. */
    private fun fetch(args: ByteArray): ByteArray {
        val (head, body) = unframe(args)
        val connection = URL(head.optString("url")).openConnection() as? HttpURLConnection
            ?: throw PlatformFailure("weft: unsupported URL ${head.optString("url")}")
        try {
            connection.requestMethod = head.optString("method", "GET")
            head.optJSONObject("headers")?.let { headers ->
                for (name in headers.keys()) connection.setRequestProperty(name, headers.getString(name))
            }
            if (body.isNotEmpty()) {
                connection.doOutput = true
                connection.outputStream.use { it.write(body) }
            }
            val status = connection.responseCode
            val response = (if (status >= 400) connection.errorStream else connection.inputStream)
                ?.use(InputStream::readBytes) ?: ByteArray(0)
            return frame(JSONObject().put("status", status), response)
        } finally {
            connection.disconnect()
        }
    }

    /** `storage.get` / `set` / `remove` on the app's shared preferences. */
    private fun storage(method: String, args: ByteArray): ByteArray {
        val array = JSONArray(args.decodeToString())
        val key = array.optString(0, null) ?: throw PlatformFailure("weft: storage.$method expects a JSON array of strings")
        when (method) {
            "get" -> return (preferences.getString(key, null)?.let(JSONObject::quote) ?: "null").encodeToByteArray()
            "set" -> {
                if (array.length() != 2) throw PlatformFailure("weft: storage.set expects [key, value]")
                preferences.edit().putString(key, array.getString(1)).apply()
            }
            else -> preferences.edit().remove(key).apply()
        }
        return ByteArray(0)
    }

    companion object {
        val SUPPORTED = setOf("fetch", "storage")

        /** Name of the shared preferences of `storage`. */
        private const val STORAGE = "weft.storage"

        /** Threads of platform calls. */
        private val background: ExecutorService = Executors.newCachedThreadPool()

        /** Splits `u32 head length (LE) | head (UTF-8 JSON) | body`. */
        private fun unframe(data: ByteArray): Pair<JSONObject, ByteArray> {
            if (data.size < 4) throw PlatformFailure("weft: truncated frame")
            val length = (0 until 4).fold(0L) { acc, i -> acc or ((data[i].toLong() and 0xff) shl (8 * i)) }
            if (length > data.size - 4) throw PlatformFailure("weft: invalid frame head")
            val head = JSONObject(data.decodeToString(4, 4 + length.toInt()))
            return head to data.copyOfRange(4 + length.toInt(), data.size)
        }

        private fun frame(head: JSONObject, body: ByteArray): ByteArray {
            val json = head.toString().encodeToByteArray()
            val out = ByteArray(4 + json.size + body.size)
            for (i in 0 until 4) out[i] = (json.size ushr (8 * i)).toByte()
            json.copyInto(out, 4)
            body.copyInto(out, 4 + json.size)
            return out
        }
    }
}

/** A failed platform call; its message rejects the call's promise. */
internal class PlatformFailure(message: String) : IOException(message)
