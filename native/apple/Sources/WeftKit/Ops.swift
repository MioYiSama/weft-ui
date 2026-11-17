/// One view protocol op (docs/PROTOCOL.md, "Ops").
enum Op: Equatable {
    case create(scope: UInt32, template: UInt32)
    case set(scope: UInt32, slot: UInt32, value: WeftValue)
    case branch(scope: UInt32, point: UInt32, child: UInt32)
    case rows(scope: UInt32, point: UInt32, children: [UInt32])
    case attach(scope: UInt32, point: UInt32, child: UInt32)
}

/// Why a batch of ops could not be decoded.
enum OpError: Error, Equatable {
    case version(UInt8)
    case truncated
    case opcode(UInt8)
    case tag(UInt8)
}

/// Decodes one batch of ops in the binary encoding (version 1).
struct OpReader {
    private let bytes: UnsafeRawBufferPointer
    private var offset = 1

    /// The encoding version of `bytes`.
    static let version: UInt8 = 1

    init(_ bytes: UnsafeRawBufferPointer) throws(OpError) {
        guard let version = bytes.first else { throw .truncated }
        guard version == Self.version else { throw .version(version) }
        self.bytes = bytes
    }

    /// The next op, or `nil` at the end of the batch.
    mutating func next() throws(OpError) -> Op? {
        guard offset < bytes.count else { return nil }
        let opcode = try byte()
        switch opcode {
        case 1:
            return .create(scope: try u32(), template: try u32())
        case 2:
            return .set(scope: try u32(), slot: try u32(), value: try value())
        case 3:
            return .branch(scope: try u32(), point: try u32(), child: try u32())
        case 4:
            let scope = try u32(), point = try u32(), count = try u32()
            guard Int(count) <= (bytes.count - offset) / 4 else { throw .truncated }
            var children: [UInt32] = []
            children.reserveCapacity(Int(count))
            for _ in 0..<count { children.append(try u32()) }
            return .rows(scope: scope, point: point, children: children)
        case 5:
            return .attach(scope: try u32(), point: try u32(), child: try u32())
        default:
            throw .opcode(opcode)
        }
    }

    private mutating func take(_ count: Int) throws(OpError) -> UnsafeRawBufferPointer {
        guard count <= bytes.count - offset else { throw .truncated }
        defer { offset += count }
        return UnsafeRawBufferPointer(rebasing: bytes[offset..<offset + count])
    }

    private mutating func byte() throws(OpError) -> UInt8 {
        try take(1)[0]
    }

    private mutating func u32() throws(OpError) -> UInt32 {
        UInt32(littleEndian: try take(4).loadUnaligned(as: UInt32.self))
    }

    private mutating func value() throws(OpError) -> WeftValue {
        let tag = try byte()
        switch tag {
        case 0: return .bool(false)
        case 1: return .bool(true)
        case 2: return .number(Double(bitPattern: UInt64(littleEndian: try take(8).loadUnaligned(as: UInt64.self))))
        case 3: return .string(String(decoding: try take(Int(try u32())), as: UTF8.self))
        default: throw .tag(tag)
        }
    }
}
