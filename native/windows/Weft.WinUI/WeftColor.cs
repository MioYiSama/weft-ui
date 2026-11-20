using System;
using System.Collections.Generic;
using System.Globalization;
using Microsoft.UI.Xaml;
using Windows.UI;

namespace Weft.WinUI;

/// <summary>A fixed RGBA color or a named token in the app's theme.</summary>
public readonly record struct WeftColor
{
    private readonly uint rgba;
    private readonly string? token;

    private WeftColor(uint rgba, string? token) => (this.rgba, this.token) = (rgba, token);

    /// <summary>A fixed color, written as <c>0xRRGGBBAA</c>.</summary>
    public static WeftColor Rgba(uint rgba) => new(rgba, null);

    /// <summary>A reference to a named theme color.</summary>
    public static WeftColor Token(string name) => new(0, name);

    /// <summary>Parses a hexadecimal color or <c>@name</c>; invalid values become transparent.</summary>
    public static WeftColor Parse(string text)
    {
        if (TryParse(text, out var color)) return color;
#if DEBUG
        WeftLog.Write(Native.LogWarn, $"weft: invalid color {text}, using transparent");
#endif
        return default;
    }

    internal static bool TryParse(string text, out WeftColor color)
    {
        color = default;
        if (text.Length > 1 && text[0] == '@' && (char.IsAsciiLetter(text[1]) || text[1] == '_'))
        {
            foreach (var c in text.AsSpan(2))
            {
                if (!char.IsAsciiLetterOrDigit(c) && c != '_') return false;
            }
            color = Token(text[1..]);
            return true;
        }
        if (text.Length is not (4 or 7 or 9) || text[0] != '#') return false;
        var digits = text.AsSpan(1);
        if (!uint.TryParse(digits, NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture, out var value))
            return false;
        // Number parsing accepts trailing nulls; color literals permit only hex digits.
        if (digits[^1] == '\0') return false;
        color = Rgba(digits.Length switch
        {
            3 => (value >> 8 & 0xf) * 17 << 24 | (value >> 4 & 0xf) * 17 << 16 | (value & 0xf) * 17 << 8 | 0xff,
            6 => value << 8 | 0xff,
            _ => value,
        });
        return true;
    }

    internal bool IsToken => token is not null;

    internal Color Resolve(WeftTheme? theme, ElementTheme appearance)
    {
        if (token is null) return ToColor(rgba);
        if (theme is not null && theme.Colors.TryGetValue(token, out var pair))
            return ToColor(appearance == ElementTheme.Dark ? pair.Dark : pair.Light);
#if DEBUG
        WeftLog.Write(Native.LogWarn, $"weft: unknown theme token {token}, using transparent");
#endif
        return default;
    }

    private static Color ToColor(uint rgba) =>
        Color.FromArgb((byte)rgba, (byte)(rgba >> 24), (byte)(rgba >> 16), (byte)(rgba >> 8));
}

/// <summary>The color tokens registered for a single Weft session.</summary>
public sealed class WeftTheme
{
    internal IReadOnlyDictionary<string, (uint Light, uint Dark)> Colors { get; }

    private WeftTheme(Dictionary<string, (uint Light, uint Dark)> colors) => Colors = colors;

    /// <summary>Registers the generated token table before starting the session.</summary>
    public static WeftTheme Register(Dictionary<string, (uint Light, uint Dark)> colors) => new(colors);
}
