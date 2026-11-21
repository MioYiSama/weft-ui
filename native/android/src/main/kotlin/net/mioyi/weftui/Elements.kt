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

enum class WeftShadow {
    NONE,
    SMALL,
    MEDIUM,
    LARGE;

    companion object {
        fun parse(value: String): WeftShadow = when (value) {
            "small" -> SMALL
            "medium" -> MEDIUM
            "large" -> LARGE
            else -> NONE
        }
    }
}

enum class WeftWeight {
    REGULAR,
    LIGHT,
    MEDIUM,
    SEMIBOLD,
    BOLD;

    companion object {
        fun parse(value: String): WeftWeight = when (value) {
            "light" -> LIGHT
            "medium" -> MEDIUM
            "semibold" -> SEMIBOLD
            "bold" -> BOLD
            else -> REGULAR
        }
    }
}

enum class WeftTextAlign {
    START,
    CENTER,
    END;

    companion object {
        fun parse(value: String): WeftTextAlign = when (value) {
            "center" -> CENTER
            "end" -> END
            else -> START
        }
    }
}

enum class WeftDecoration {
    NONE,
    UNDERLINE,
    LINE_THROUGH;

    companion object {
        fun parse(value: String): WeftDecoration = when (value) {
            "underline" -> UNDERLINE
            "lineThrough" -> LINE_THROUGH
            else -> NONE
        }
    }
}

enum class WeftInputType {
    TEXT,
    PASSWORD,
    NUMBER,
    EMAIL,
    PHONE;

    companion object {
        fun parse(value: String): WeftInputType = when (value) {
            "password" -> PASSWORD
            "number" -> NUMBER
            "email" -> EMAIL
            "phone" -> PHONE
            else -> TEXT
        }
    }
}

enum class WeftReturnKey {
    DONE,
    GO,
    NEXT,
    SEARCH,
    SEND;

    companion object {
        fun parse(value: String): WeftReturnKey = when (value) {
            "go" -> GO
            "next" -> NEXT
            "search" -> SEARCH
            "send" -> SEND
            else -> DONE
        }
    }
}

enum class WeftImageFit {
    COVER,
    CONTAIN,
    FILL;

    companion object {
        fun parse(value: String): WeftImageFit = when (value) {
            "contain" -> CONTAIN
            "fill" -> FILL
            else -> COVER
        }
    }
}
