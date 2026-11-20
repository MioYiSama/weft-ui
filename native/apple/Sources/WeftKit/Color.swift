import SwiftUI
#if os(iOS)
import UIKit
private typealias NativeColor = UIColor
#elseif os(macOS)
import AppKit
private typealias NativeColor = NSColor
#endif

/// A fixed sRGB color (`0xRRGGBBAA`) or a reference to the app's theme.
public enum WeftColor: Equatable, Sendable {
    case rgba(UInt32)
    case token(String)

    /// Transparent black, also used for invalid colors.
    public static let clear = WeftColor.rgba(0)

    /// Parses `#RGB`, `#RRGGBB`, `#RRGGBBAA`, or `@name` from a value slot.
    public static func parse(_ text: String) -> Self {
        let bytes = text.utf8
        if bytes.first == UInt8(ascii: "#"), let rgba = hexRGBA(bytes.dropFirst()) {
            return .rgba(rgba)
        }
        if bytes.first == UInt8(ascii: "@") {
            let name = bytes.dropFirst()
            if let first = name.first, isIdentifierStart(first),
               name.dropFirst().allSatisfy({ isIdentifierStart($0) || (48...57).contains($0) }) {
                return .token(String(text.dropFirst()))
            }
        }
        #if DEBUG
        logger.warning("weft: invalid color \(text, privacy: .public), using transparent")
        #endif
        return .clear
    }

    /// Returns the fixed color or a native dynamic color that follows system appearance.
    @MainActor public var resolved: Color {
        switch self {
        case .rgba(let rgba):
            return Color(nativeColor(rgba))
        case .token(let name):
            if let color = WeftTheme.colors[name] { return color }
            #if DEBUG
            logger.warning("weft: unknown theme token \(name, privacy: .public), using transparent")
            #endif
            return .clear
        }
    }

    private static func isIdentifierStart(_ byte: UInt8) -> Bool {
        (65...90).contains(byte) || (97...122).contains(byte) || byte == UInt8(ascii: "_")
    }

    private static func hexRGBA(_ digits: String.UTF8View.SubSequence) -> UInt32? {
        let count = digits.count
        guard count == 3 || count == 6 || count == 8 else { return nil }
        var value: UInt32 = 0
        for byte in digits {
            let digit: UInt8
            switch byte {
            case 48...57: digit = byte - 48
            case 65...70: digit = byte - 65 + 10
            case 97...102: digit = byte - 97 + 10
            default: return nil
            }
            value = value << 4 | UInt32(digit)
        }
        if count == 3 {
            let r = value >> 8, g = value >> 4 & 0xf, b = value & 0xf
            return (r * 17) << 24 | (g * 17) << 16 | (b * 17) << 8 | 0xff
        }
        return count == 6 ? value << 8 | 0xff : value
    }
}

/// The generated app registers its light and dark RGBA pairs before starting the kernel.
@MainActor public enum WeftTheme {
    fileprivate static var colors: [String: Color] = [:]

    /// Replaces the theme palette with native colors resolved by SwiftUI at render time.
    public static func register(_ colors: [String: (light: UInt32, dark: UInt32)]) {
        self.colors = colors.mapValues { pair in
            let light = nativeColor(pair.light)
            if pair.light == pair.dark { return Color(light) }
            let dark = nativeColor(pair.dark)
            #if os(iOS)
            return Color(uiColor: UIColor { traits in
                traits.userInterfaceStyle == .dark ? dark : light
            })
            #elseif os(macOS)
            return Color(nsColor: NSColor(name: nil) { appearance in
                appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? dark : light
            })
            #endif
        }
    }
}

private func nativeColor(_ rgba: UInt32) -> NativeColor {
    let red = CGFloat(rgba >> 24 & 0xff) / 255
    let green = CGFloat(rgba >> 16 & 0xff) / 255
    let blue = CGFloat(rgba >> 8 & 0xff) / 255
    let alpha = CGFloat(rgba & 0xff) / 255
    #if os(iOS)
    return UIColor(red: red, green: green, blue: blue, alpha: alpha)
    #elseif os(macOS)
    return NSColor(srgbRed: red, green: green, blue: blue, alpha: alpha)
    #endif
}
