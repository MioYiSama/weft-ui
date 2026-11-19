import Foundation

/// The platform modules of the shell; byte formats in
/// docs/src/content/docs/protocol.md ("Platform calls").
enum PlatformModules {
    static let supported: Set<String> = ["fetch", "storage"]

    static func call(_ module: String, _ method: String, _ args: Data) async throws -> Data {
        switch (module, method) {
        case ("fetch", "request"):
            try await fetch(args)
        case ("storage", "get"), ("storage", "set"), ("storage", "remove"):
            try storage(method, args)
        default:
            throw PlatformFailure("weft: \(module).\(method) is not implemented")
        }
    }

    /// `fetch.request` through URLSession.
    private static func fetch(_ args: Data) async throws -> Data {
        let (head, body) = try unframe(args)
        guard let text = head["url"] as? String, let url = URL(string: text) else {
            throw PlatformFailure("weft: invalid URL \(head["url"] ?? "")")
        }
        var request = URLRequest(url: url)
        request.httpMethod = head["method"] as? String ?? "GET"
        for (name, value) in head["headers"] as? [String: String] ?? [:] {
            request.setValue(value, forHTTPHeaderField: name)
        }
        if !body.isEmpty {
            request.httpBody = body
        }
        let (data, response) = try await URLSession.shared.data(for: request)
        // Responses without a status (`data:` and `file:` URLs) succeeded.
        let status = (response as? HTTPURLResponse)?.statusCode ?? 200
        return try frame(["status": status], data)
    }

    /// `storage.get` / `set` / `remove` on the standard user defaults.
    private static func storage(_ method: String, _ args: Data) throws -> Data {
        guard let args = try JSONSerialization.jsonObject(with: args) as? [String], let key = args.first else {
            throw PlatformFailure("weft: storage.\(method) expects a JSON array of strings")
        }
        let defaults = UserDefaults.standard
        let name = "weft.storage.\(key)"
        switch method {
        case "get":
            guard let value = defaults.string(forKey: name) else { return Data("null".utf8) }
            return try JSONSerialization.data(withJSONObject: value, options: .fragmentsAllowed)
        case "set":
            guard args.count == 2 else { throw PlatformFailure("weft: storage.set expects [key, value]") }
            defaults.set(args[1], forKey: name)
        default:
            defaults.removeObject(forKey: name)
        }
        return Data()
    }

    /// Splits `u32 head length (LE) | head (UTF-8 JSON) | body`.
    static func unframe(_ data: Data) throws -> (head: [String: Any], body: Data) {
        let bytes = [UInt8](data)
        guard bytes.count >= 4 else { throw PlatformFailure("weft: truncated frame") }
        let length = Int(UInt32(bytes[0]) | UInt32(bytes[1]) << 8 | UInt32(bytes[2]) << 16 | UInt32(bytes[3]) << 24)
        guard length <= bytes.count - 4,
              let head = try JSONSerialization.jsonObject(with: Data(bytes[4..<4 + length])) as? [String: Any]
        else { throw PlatformFailure("weft: invalid frame head") }
        return (head, Data(bytes[(4 + length)...]))
    }

    static func frame(_ head: [String: Any], _ body: Data) throws -> Data {
        let json = try JSONSerialization.data(withJSONObject: head)
        var data = Data()
        withUnsafeBytes(of: UInt32(json.count).littleEndian) { data.append(contentsOf: $0) }
        data.append(json)
        data.append(body)
        return data
    }
}

/// A failed platform call; its description rejects the call's promise.
struct PlatformFailure: LocalizedError {
    let errorDescription: String?

    init(_ message: String) {
        errorDescription = message
    }
}
