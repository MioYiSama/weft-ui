import SwiftUI

/// A value of a value slot or an event payload (docs/src/content/docs/protocol.md, "Binary encoding").
public enum WeftValue: Equatable, Sendable {
    case bool(Bool)
    case number(Double)
    case string(String)

    /// The value of a `String` slot.
    public var string: String {
        if case .string(let value) = self { value } else { "" }
    }

    /// The value of a `Bool` slot.
    public var bool: Bool {
        if case .bool(let value) = self { value } else { false }
    }

    /// The value of an `Int` slot: a length in points.
    public var length: CGFloat {
        if case .number(let value) = self { CGFloat(value) } else { 0 }
    }

    /// The value of a `Color` slot; invalid colors are transparent.
    public var color: Color {
        if let color = Color(weftHex: string) { return color }
        #if DEBUG
        logger.warning("weft: invalid color \(string, privacy: .public), using transparent")
        #endif
        return .clear
    }

    /// Appends the binary encoding: a tag byte and the data.
    func encode(into bytes: inout [UInt8]) {
        switch self {
        case .bool(let value):
            bytes.append(value ? 1 : 0)
        case .number(let value):
            bytes.append(2)
            withUnsafeBytes(of: value.bitPattern.littleEndian) { bytes.append(contentsOf: $0) }
        case .string(let value):
            bytes.append(3)
            let utf8 = Array(value.utf8)
            withUnsafeBytes(of: UInt32(utf8.count).littleEndian) { bytes.append(contentsOf: $0) }
            bytes.append(contentsOf: utf8)
        }
    }
}

extension Color {
    /// A constant color of the generated views, written as `0xRRGGBBAA`.
    public init(weftRGBA rgba: UInt32) {
        self.init(
            .sRGB,
            red: Double(rgba >> 24 & 0xff) / 255,
            green: Double(rgba >> 16 & 0xff) / 255,
            blue: Double(rgba >> 8 & 0xff) / 255,
            opacity: Double(rgba & 0xff) / 255
        )
    }

    /// Parses `#RGB`, `#RRGGBB` or `#RRGGBBAA` in either hex case.
    init?(weftHex text: String) {
        let digits = Array(text.utf8.dropFirst())
        guard text.utf8.first == UInt8(ascii: "#"),
              [3, 6, 8].contains(digits.count),
              digits.allSatisfy({ Character(Unicode.Scalar($0)).isHexDigit }),
              let value = UInt32(String(decoding: digits, as: UTF8.self), radix: 16)
        else { return nil }
        switch digits.count {
        case 3:
            let r = value >> 8 & 0xf, g = value >> 4 & 0xf, b = value & 0xf
            self.init(weftRGBA: (r * 17) << 24 | (g * 17) << 16 | (b * 17) << 8 | 0xff)
        case 6:
            self.init(weftRGBA: value << 8 | 0xff)
        default:
            self.init(weftRGBA: value)
        }
    }
}
