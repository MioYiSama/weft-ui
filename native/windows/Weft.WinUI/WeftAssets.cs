using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace Weft.WinUI;

// The Vite plugin writes index.json with image variants in ascending scale order.
internal sealed record WeftAssets(Dictionary<string, WeftAssets.ImageVariant[]> Images, WeftAssets.FontVariant[] Fonts)
{
    internal const string UriPrefix = "ms-appx:///weft/assets/";
    private static readonly Lazy<WeftAssets?> Bundled = new(() => Read(Path.Combine(AppContext.BaseDirectory, "weft", "assets")));
    internal static WeftAssets? Current => Bundled.Value;

    public sealed record ImageVariant(string File, int Scale)
    {
        internal bool IsSvg => File.EndsWith(".svg", StringComparison.Ordinal);
    }

    public sealed record FontVariant(string File, string Family, ushort Weight, bool Italic)
    {
        internal string Source => $"{UriPrefix}fonts/{Uri.EscapeDataString(File)}#{Family}";
    }

    internal static WeftAssets? Read(string directory)
    {
        var path = Path.Combine(directory, "index.json");
        if (!File.Exists(path)) return null;
        using var input = File.OpenRead(path);
        return JsonSerializer.Deserialize<WeftAssets>(input);
    }

    internal ImageVariant? FindImage(string name, double scale)
    {
        if (!Images.TryGetValue(name, out var variants) || variants.Length == 0) return null;
        foreach (var variant in variants)
            if (variant.Scale >= scale) return variant;
        return variants[^1];
    }

    internal FontVariant? FindFont(string family, ushort weight, bool italic)
    {
        FontVariant? best = null;
        var rank = int.MaxValue;
        foreach (var font in Fonts)
        {
            if (!string.Equals(font.Family, family, StringComparison.OrdinalIgnoreCase)) continue;
            var next = WeightRank(weight, font.Weight) + (font.Italic == italic ? 0 : 2000);
            if (next == 0) return font;
            if (next >= rank) continue;
            best = font;
            rank = next;
        }
        return best;
    }

    // Match CSS: at 400–500 prefer weights up to 500, then lighter, then heavier.
    // Outside that range search toward the nearer end of the weight range first.
    private static int WeightRank(int weight, int candidate) => weight switch
    {
        < 400 => candidate <= weight ? weight - candidate : 1000 + candidate - weight,
        > 500 => candidate >= weight ? candidate - weight : 1000 + weight - candidate,
        _ => candidate >= weight && candidate <= 500 ? candidate - weight
            : candidate < weight ? 500 - candidate : 500 + candidate,
    };
}
