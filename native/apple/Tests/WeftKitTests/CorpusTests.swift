import Foundation
import Testing
@testable import WeftKit

private typealias JSONObject = [String: Any]

private func corpus(_ file: String) throws -> JSONObject {
    var root = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { root.deleteLastPathComponent() }
    let url = root.appendingPathComponent("protocol/corpus/\(file).json")
    return try #require(JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? JSONObject)
}

private func hexBytes(_ test: JSONObject) throws -> [UInt8] {
    let hex = try #require(test["hex"] as? String).filter { !$0.isWhitespace }
    var bytes: [UInt8] = []
    bytes.reserveCapacity(hex.count / 2)
    var start = hex.startIndex
    while start < hex.endIndex {
        let end = try #require(hex.index(start, offsetBy: 2, limitedBy: hex.endIndex))
        bytes.append(try #require(UInt8(hex[start..<end], radix: 16)))
        start = end
    }
    return bytes
}

private func expectedValue(_ json: Any?) throws -> WeftValue {
    let value = try #require(json as? JSONObject)
    if let bool = value["bool"] as? Bool { return .bool(bool) }
    if let number = value["number"] as? Double { return .number(number) }
    return .string(try #require(value["string"] as? String))
}

private func expectedOp(_ json: JSONObject) throws -> Op {
    func field(_ name: String) throws -> UInt32 {
        try #require(json[name] as? UInt32)
    }
    let op = try #require(json["op"] as? String)
    let scope = try field("scope")
    switch op {
    case "create": return .create(scope: scope, template: try field("template"))
    case "set": return .set(scope: scope, slot: try field("slot"), value: try expectedValue(json["value"]))
    case "branch": return .branch(scope: scope, point: try field("point"), child: try field("child"))
    case "rows": return .rows(scope: scope, point: try field("point"), children: try #require(json["children"] as? [UInt32]))
    case "attach": return .attach(scope: scope, point: try field("point"), child: try field("child"))
    default: throw DecodingError.dataCorrupted(.init(codingPath: [], debugDescription: "Unknown corpus op: \(op)"))
    }
}

private func decode(_ bytes: [UInt8]) throws -> [Op] {
    try bytes.withUnsafeBytes(OpReader.readAll)
}

@Test func decodesOpsCorpus() throws {
    let corpus = try corpus("ops")
    for test in try #require(corpus["cases"] as? [JSONObject]) {
        let name = try #require(test["name"] as? String)
        let bytes = try hexBytes(test)
        if let kind = test["error"] as? String {
            do {
                _ = try decode(bytes)
                Issue.record("\(name): expected \(kind)")
            } catch let error as OpError {
                #expect(error.kind == kind, "\(name)")
            }
        } else {
            let expected = try #require(test["ops"] as? [JSONObject]).map(expectedOp)
            #expect(try decode(bytes) == expected, "\(name)")
        }
    }
}

@Test @MainActor func malformedOpsDoNotCreateScopes() throws {
    let cases = try #require(corpus("ops")["cases"] as? [JSONObject])
    let test = try #require(cases.first { $0["name"] as? String == "unknown-opcode-after-create" })
    let bytes = try hexBytes(test)
    var createdScopes: [UInt32] = []
    let session = WeftSession(bytecode: nil, layoutHash: 0) { _, id, session in
        createdScopes.append(id)
        return WeftScope(id: id, session: session)
    }

    bytes.withUnsafeBytes { session.commit($0) }

    #expect(session.failure?.hasPrefix("weft: malformed ops") == true)
    #expect(createdScopes.isEmpty)
}

@Test func encodesValuesCorpus() throws {
    let corpus = try corpus("values")
    for test in try #require(corpus["cases"] as? [JSONObject]) {
        let name = try #require(test["name"] as? String)
        var bytes: [UInt8] = []
        try expectedValue(test["value"]).encode(into: &bytes)
        #expect(try bytes == hexBytes(test), "\(name)")
    }
}

// Compare header JSON without depending on object key order or input formatting.
private func canonicalJSON(_ json: JSONObject) throws -> Data {
    try JSONSerialization.data(withJSONObject: json, options: .sortedKeys)
}

@Test func platformFramesMatchCorpus() throws {
    let corpus = try corpus("frames")
    for test in try #require(corpus["requests"] as? [JSONObject]) {
        let name = try #require(test["name"] as? String)
        let expectedHead = try #require(test["head"] as? JSONObject)
        let expectedBody = Data(try #require(test["body"] as? String).utf8)
        let (head, body) = try PlatformModules.unframe(Data(hexBytes(test)))
        #expect(try canonicalJSON(head) == canonicalJSON(expectedHead), "\(name)")
        #expect(body == expectedBody, "\(name)")
    }
    for test in try #require(corpus["responses"] as? [JSONObject]) {
        let name = try #require(test["name"] as? String)
        let expectedHead = try #require(test["head"] as? JSONObject)
        let expectedBody = Data(try #require(test["body"] as? String).utf8)
        let frame = try PlatformModules.frame(expectedHead, expectedBody)
        try #require(frame.count >= 4, "\(name)")
        let length = Int(UInt32(littleEndian: frame.withUnsafeBytes { $0.loadUnaligned(as: UInt32.self) }))
        try #require(length <= frame.count - 4, "\(name)")
        let head = try #require(JSONSerialization.jsonObject(with: frame[4..<4 + length]) as? JSONObject, "\(name)")
        #expect(try canonicalJSON(head) == canonicalJSON(expectedHead), "\(name)")
        #expect(frame.dropFirst(4 + length) == expectedBody, "\(name)")
    }
    for test in try #require(corpus["malformed"] as? [JSONObject]) {
        let name = try #require(test["name"] as? String)
        let frame = try Data(hexBytes(test))
        #expect(throws: (any Error).self, "\(name)") { try PlatformModules.unframe(frame) }
    }
}
