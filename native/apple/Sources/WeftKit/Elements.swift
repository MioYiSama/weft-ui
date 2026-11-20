// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

import Foundation

public enum WeftCrossAlign: String {
    case start
    case center
    case end
    case stretch

    public init(weft: String) {
        if let value = Self(rawValue: weft) {
            self = value
        } else {
            #if DEBUG
            NSLog("weft: invalid CrossAlign value: %@", weft)
            #endif
            self = .start
        }
    }
}

public enum WeftJustify: String {
    case start
    case center
    case end
    case spaceBetween
    case spaceAround
    case spaceEvenly

    public init(weft: String) {
        if let value = Self(rawValue: weft) {
            self = value
        } else {
            #if DEBUG
            NSLog("weft: invalid Justify value: %@", weft)
            #endif
            self = .start
        }
    }
}

public enum WeftAlignment: String {
    case topStart
    case top
    case topEnd
    case start
    case center
    case end
    case bottomStart
    case bottom
    case bottomEnd

    public init(weft: String) {
        if let value = Self(rawValue: weft) {
            self = value
        } else {
            #if DEBUG
            NSLog("weft: invalid Alignment value: %@", weft)
            #endif
            self = .topStart
        }
    }
}

public enum WeftAxis: String {
    case vertical
    case horizontal

    public init(weft: String) {
        if let value = Self(rawValue: weft) {
            self = value
        } else {
            #if DEBUG
            NSLog("weft: invalid Axis value: %@", weft)
            #endif
            self = .vertical
        }
    }
}
