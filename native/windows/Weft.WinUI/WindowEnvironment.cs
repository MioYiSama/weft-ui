using System.Buffers;
using System.Text.Json;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;

namespace Weft.WinUI;

internal readonly record struct WindowEnvironment(bool Dark, double Width, double Height, double Scale)
{
    internal static WindowEnvironment Initial()
    {
        var area = DisplayArea.Primary?.WorkArea ?? default;
        var scale = Native.GetDpiForSystem() / 96.0;
        return new(Application.Current.RequestedTheme == ApplicationTheme.Dark,
            area.Width / scale, area.Height / scale, scale);
    }

    internal void WriteJson(ArrayBufferWriter<byte> buffer)
    {
        buffer.Clear();
        using var json = new Utf8JsonWriter(buffer);
        json.WriteStartObject();
        json.WriteString("colorScheme", Dark ? "dark" : "light");
        json.WriteStartObject("screen");
        json.WriteNumber("width", Width);
        json.WriteNumber("height", Height);
        json.WriteNumber("scale", Scale);
        json.WriteEndObject();
        json.WriteStartObject("safeArea");
        json.WriteNumber("top", 0);
        json.WriteNumber("bottom", 0);
        json.WriteNumber("start", 0);
        json.WriteNumber("end", 0);
        json.WriteEndObject();
        json.WriteString("orientation", Width > Height ? "landscape" : "portrait");
        json.WriteEndObject();
    }
}
