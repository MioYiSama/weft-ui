package dev.weft

import android.util.Log

/** Logcat output of the shell and of the app's `console`, tagged `weft`. */
internal object WeftLog {
    private const val TAG = "weft"

    /** Whether the app is a debug build; set when the session starts. */
    @Volatile
    var debuggable = false

    fun write(level: Int, message: String) {
        val priority = when (level) {
            LogLevel.DEBUG -> Log.DEBUG
            LogLevel.INFO -> Log.INFO
            LogLevel.WARN -> Log.WARN
            else -> Log.ERROR
        }
        Log.println(priority, TAG, message)
    }

    fun warn(message: String) = write(LogLevel.WARN, message)

    fun error(message: String) = write(LogLevel.ERROR, message)
}
