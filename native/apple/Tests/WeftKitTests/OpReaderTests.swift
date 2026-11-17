import Testing
@testable import WeftKit

private func u32(_ value: UInt32) -> [UInt8] {
    withUnsafeBytes(of: value.littleEndian) { Array($0) }
}

private func decode(_ bytes: [UInt8]) throws -> [Op] {
    try bytes.withUnsafeBytes { raw in
        var reader = try OpReader(raw)
        var ops: [Op] = []
        while let op = try reader.next() { ops.append(op) }
        return ops
    }
}

@Test func decodesEveryOpAndValue() throws {
    var string: [UInt8] = []
    WeftValue.string("mïlk").encode(into: &string)
    var number: [UInt8] = []
    WeftValue.number(-2.5).encode(into: &number)
    let bytes: [UInt8] = [1]
        + [1] + u32(3) + u32(2)
        + [2] + u32(3) + u32(0) + string
        + [2] + u32(3) + u32(1) + number
        + [2] + u32(3) + u32(2) + [1]
        + [4] + u32(2) + u32(0) + u32(2) + u32(3) + u32(4)
        + [3] + u32(1) + u32(0) + u32(0)
        + [5] + u32(0) + u32(0) + u32(1)
    #expect(try decode(bytes) == [
        .create(scope: 3, template: 2),
        .set(scope: 3, slot: 0, value: .string("mïlk")),
        .set(scope: 3, slot: 1, value: .number(-2.5)),
        .set(scope: 3, slot: 2, value: .bool(true)),
        .rows(scope: 2, point: 0, children: [3, 4]),
        .branch(scope: 1, point: 0, child: 0),
        .attach(scope: 0, point: 0, child: 1),
    ])
}

@Test func rejectsMalformedBatches() {
    #expect(throws: OpError.version(2)) { try decode([2]) }
    #expect(throws: OpError.truncated) { try decode([1, 1] + u32(3)) }
    // A row count larger than the rest of the batch.
    #expect(throws: OpError.truncated) { try decode([1, 4] + u32(2) + u32(0) + u32(1000)) }
    #expect(throws: OpError.opcode(9)) { try decode([1, 9]) }
    #expect(throws: OpError.tag(7)) { try decode([1, 2] + u32(1) + u32(0) + [7]) }
}
