// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

package net.mioyi.weftui

enum class WeftCrossAlign {
    START,
    CENTER,
    END,
    STRETCH;

    companion object {
        fun parse(value: String): WeftCrossAlign = when (value) {
            "center" -> CENTER
            "end" -> END
            "stretch" -> STRETCH
            else -> START
        }
    }
}

enum class WeftJustify {
    START,
    CENTER,
    END,
    SPACE_BETWEEN,
    SPACE_AROUND,
    SPACE_EVENLY;

    companion object {
        fun parse(value: String): WeftJustify = when (value) {
            "center" -> CENTER
            "end" -> END
            "spaceBetween" -> SPACE_BETWEEN
            "spaceAround" -> SPACE_AROUND
            "spaceEvenly" -> SPACE_EVENLY
            else -> START
        }
    }
}

enum class WeftAlignment {
    TOP_START,
    TOP,
    TOP_END,
    START,
    CENTER,
    END,
    BOTTOM_START,
    BOTTOM,
    BOTTOM_END;

    companion object {
        fun parse(value: String): WeftAlignment = when (value) {
            "top" -> TOP
            "topEnd" -> TOP_END
            "start" -> START
            "center" -> CENTER
            "end" -> END
            "bottomStart" -> BOTTOM_START
            "bottom" -> BOTTOM
            "bottomEnd" -> BOTTOM_END
            else -> TOP_START
        }
    }
}

enum class WeftAxis {
    VERTICAL,
    HORIZONTAL;

    companion object {
        fun parse(value: String): WeftAxis = when (value) {
            "horizontal" -> HORIZONTAL
            else -> VERTICAL
        }
    }
}
