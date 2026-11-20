import SwiftUI

/// A value of a value slot or an event payload (protocol/SPEC.md, "Binary encoding").
public enum WeftValue: Equatable, Sendable {
    case bool(Bool)
    case number(Double)
    case string(String)
    case strings([String])

    /// The value of a `String` slot.
    public var string: String {
        if case .string(let value) = self { value } else { "" }
    }

    /// The value of a `Strings` slot.
    public var strings: [String] {
        if case .strings(let value) = self { value } else { [] }
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
    public var color: WeftColor {
        WeftColor.parse(string)
    }

    /// Appends the binary encoding of an event payload: a tag byte and the data.
    func encode(into bytes: inout [UInt8]) {
        switch self {
        case .bool(let value):
            bytes.append((value ? ValueTag.true : .false).rawValue)
        case .number(let value):
            bytes.append(ValueTag.number.rawValue)
            withUnsafeBytes(of: value.bitPattern.littleEndian) { bytes.append(contentsOf: $0) }
        case .string(let value):
            bytes.append(ValueTag.string.rawValue)
            let utf8 = Array(value.utf8)
            withUnsafeBytes(of: UInt32(utf8.count).littleEndian) { bytes.append(contentsOf: $0) }
            bytes.append(contentsOf: utf8)
        case .strings:
            preconditionFailure("weft: string lists are not event payloads")
        }
    }
}

