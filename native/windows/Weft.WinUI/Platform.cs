using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading.Tasks;

namespace Weft.WinUI;

/// <summary>The platform modules of the shell; byte formats in docs/src/content/docs/protocol.md ("Platform calls").</summary>
internal static class PlatformModules
{
    public static readonly HashSet<string> Supported = ["fetch", "storage"];

    private static readonly HttpClient Http = new();

    public static Task<byte[]> Call(string module, string method, byte[] args) => (module, method) switch
    {
        ("fetch", "request") => Fetch(args),
        ("storage", "get" or "set" or "remove") => Task.FromResult(Storage.Call(method, args)),
        _ => throw new InvalidOperationException($"weft: {module}.{method} is not implemented"),
    };

    /// <summary><c>fetch.request</c> through <see cref="HttpClient"/>.</summary>
    private static async Task<byte[]> Fetch(byte[] args)
    {
        var (head, body) = Unframe(args);
        var url = head["url"]?.GetValue<string>();
        if (!Uri.TryCreate(url, UriKind.Absolute, out var uri)) throw new InvalidOperationException($"weft: invalid URL {url}");
        using var request = new HttpRequestMessage(new HttpMethod(head["method"]?.GetValue<string>() ?? "GET"), uri);
        if (body.Length > 0) request.Content = new ByteArrayContent(body);
        foreach (var (name, value) in head["headers"]?.AsObject() ?? new JsonObject())
        {
            var text = value?.GetValue<string>() ?? "";
            // Content headers such as Content-Type belong to the body.
            if (!request.Headers.TryAddWithoutValidation(name, text)) request.Content?.Headers.TryAddWithoutValidation(name, text);
        }
        using var response = await Http.SendAsync(request);
        var data = await response.Content.ReadAsByteArrayAsync();
        return Frame(new JsonObject { ["status"] = (int)response.StatusCode }, data);
    }

    /// <summary>Splits <c>u32 head length (LE) | head (UTF-8 JSON) | body</c>.</summary>
    private static (JsonObject Head, byte[] Body) Unframe(byte[] data)
    {
        if (data.Length < 4) throw new InvalidOperationException("weft: truncated frame");
        var length = BinaryPrimitives.ReadUInt32LittleEndian(data);
        if (length > data.Length - 4 || JsonNode.Parse(data.AsSpan(4, (int)length)) is not JsonObject head)
        {
            throw new InvalidOperationException("weft: invalid frame head");
        }
        return (head, data[(4 + (int)length)..]);
    }

    private static byte[] Frame(JsonObject head, byte[] body)
    {
        var json = Encoding.UTF8.GetBytes(head.ToJsonString());
        var data = new byte[4 + json.Length + body.Length];
        BinaryPrimitives.WriteUInt32LittleEndian(data, (uint)json.Length);
        json.CopyTo(data, 4);
        body.CopyTo(data, 4 + json.Length);
        return data;
    }
}

/// <summary>
/// <c>storage.get</c> / <c>set</c> / <c>remove</c>: a JSON object of strings in
/// <c>%LOCALAPPDATA%\&lt;app id&gt;\storage.json</c>. The app id is the
/// <c>WeftAppId</c> assembly metadata of the app (set by the project
/// template), else the app's assembly name.
/// </summary>
internal static class Storage
{
    private static Dictionary<string, string>? values;

    private static string FilePath
    {
        get
        {
            var app = Assembly.GetEntryAssembly();
            var id = app?.GetCustomAttributes<AssemblyMetadataAttribute>().FirstOrDefault(a => a.Key == "WeftAppId")?.Value;
            if (string.IsNullOrEmpty(id)) id = app?.GetName().Name ?? "Weft";
            return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), id, "storage.json");
        }
    }

    public static byte[] Call(string method, byte[] args)
    {
        var list = JsonSerializer.Deserialize<string[]>(args);
        if (list is not { Length: > 0 }) throw new InvalidOperationException($"weft: storage.{method} expects a JSON array of strings");
        var key = list[0];
        var stored = values ??= Load();
        switch (method)
        {
            case "get":
                return JsonSerializer.SerializeToUtf8Bytes(stored.GetValueOrDefault(key));
            case "set":
                if (list.Length != 2) throw new InvalidOperationException("weft: storage.set expects [key, value]");
                stored[key] = list[1];
                break;
            default:
                if (!stored.Remove(key)) return [];
                break;
        }
        Save(stored);
        return [];
    }

    private static Dictionary<string, string> Load()
    {
        try
        {
            return JsonSerializer.Deserialize<Dictionary<string, string>>(File.ReadAllBytes(FilePath)) ?? [];
        }
        catch (Exception error) when (error is IOException or JsonException or UnauthorizedAccessException)
        {
            return [];
        }
    }

    /// <summary>Writes the file through a temporary one, so a crash never leaves it half written.</summary>
    private static void Save(Dictionary<string, string> stored)
    {
        var path = FilePath;
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var temporary = path + ".tmp";
        File.WriteAllBytes(temporary, JsonSerializer.SerializeToUtf8Bytes(stored));
        File.Move(temporary, path, overwrite: true);
    }
}
