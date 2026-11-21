import CoreText
import SwiftUI
#if os(iOS)
import UIKit
#else
import AppKit
#endif

/// An inline text run. Unset styles inherit from the containing `Text`.
public struct WeftSpan {
    let value: String
    let color: WeftColor?
    let size: CGFloat?
    let family: String?
    let weight: WeftWeight?
    let italic: Bool?
    let decoration: WeftDecoration?

    public init(
        value: String, color: WeftColor? = nil, size: CGFloat? = nil, font: String? = nil,
        weight: WeftWeight? = nil, italic: Bool? = nil, decoration: WeftDecoration? = nil
    ) {
        self.value = value
        self.color = color
        self.size = size
        family = font
        self.weight = weight
        self.italic = italic
        self.decoration = decoration
    }
}

/// Renders plain or styled text as one SwiftUI view, including wrapping and truncation.
@MainActor public func weftText(
    spans: [WeftSpan], color: WeftColor? = nil, size: CGFloat? = nil, font: String? = nil,
    weight: WeftWeight? = nil, italic: Bool = false, decoration: WeftDecoration? = nil,
    bold: Bool? = nil, align: WeftTextAlign = .start, maxLines: CGFloat = 0, lineHeight: CGFloat? = nil
) -> some View {
    WeftText(spans: spans, color: color, size: size, family: font,
             weight: bold.map { $0 ? .bold : .regular } ?? weight, italic: italic, decoration: decoration,
             align: align, maxLines: maxLines, lineHeight: lineHeight)
}

struct WeftText: View {
    let spans: [WeftSpan]
    var color: WeftColor? = nil
    var size: CGFloat? = nil
    var family: String? = nil
    var weight: WeftWeight? = nil
    var italic = false
    var decoration: WeftDecoration? = nil
    var align: WeftTextAlign = .start
    var maxLines: CGFloat = 0
    var lineHeight: CGFloat? = nil

    @Environment(\.font) var inheritedFont
    @Environment(\.weftAssets) var assets
    var sizing = WeftSizing()

    var attributedText: AttributedString {
        let baseColor = color?.resolved
        let baseFont = font(family: family, size: size, weight: weight, italic: italic)
        var result = AttributedString()
        for span in spans where !span.value.isEmpty {
            var run = AttributedString(span.value)
            run.swiftUI.foregroundColor = span.color?.resolved ?? baseColor
            if span.family == nil && span.size == nil && span.weight == nil && span.italic == nil {
                run.swiftUI.font = baseFont
            } else {
                run.swiftUI.font = font(family: span.family ?? family, size: span.size ?? size,
                                        weight: span.weight ?? weight, italic: span.italic ?? italic)
            }
            switch span.decoration ?? decoration ?? .none {
            case .none: break
            case .underline: run.swiftUI.underlineStyle = .single
            case .lineThrough: run.swiftUI.strikethroughStyle = .single
            }
            result.append(run)
        }
        return result
    }

    var body: some View {
        let lineAlignment: TextAlignment = switch align {
        case .start: .leading
        case .center: .center
        case .end: .trailing
        }
        let text = Text(attributedText)
            .multilineTextAlignment(lineAlignment)
            .lineLimit(maxLines > 0 ? Int(exactly: maxLines) ?? Int.max : nil)
            .truncationMode(.tail)
            .lineSpacing(lineHeight.map { max(0, $0 - naturalLineHeight) } ?? 0)
        if align == .start {
            text
        } else {
            // Text aligns its lines within its own measured width, which may be smaller than its Weft box.
            let frame = sizing.frame
            let exact = sizing.exactAxes
            let exactWidth = frame.width != nil || exact.width
                || (frame.aspectRatio.map { $0 > 0 } == true && (frame.height != nil || exact.height))
            text.frame(minWidth: frame.minWidth.map { max(0, $0) }, maxWidth: exactWidth ? .infinity : nil,
                       alignment: align == .center ? .top : .topTrailing)
        }
    }

    private func face(family: String?, weight: WeftWeight?, italic: Bool) -> WeftAssets.FontFace? {
        family.flatMap { assets?.face(family: $0, weight: (weight ?? .regular).assetWeight, italic: italic) }
    }

    private func font(family: String?, size: CGFloat?, weight: WeftWeight?, italic: Bool) -> Font? {
        if let face = face(family: family, weight: weight, italic: italic) {
            // Static faces have distinct PostScript names; `.weight` on a custom font need not select them.
            let font = Font.custom(face.name, size: size ?? defaultSize)
            return italic && !face.italic ? font.italic() : font
        }
        var font = size.map { Font.system(size: $0) }
        if let weight { font = (font ?? inheritedFont ?? .body).weight(weight.swiftUI) }
        if italic { font = (font ?? inheritedFont ?? .body).italic() }
        return font
    }

    private var defaultSize: CGFloat {
        #if os(iOS)
        UIFont.preferredFont(forTextStyle: .body).pointSize
        #else
        NSFont.systemFontSize
        #endif
    }

    private var naturalLineHeight: CGFloat {
        // SwiftUI adds one spacing value to the whole paragraph, not a height per run.
        let size = size ?? defaultSize
        func height(_ span: WeftSpan?) -> CGFloat {
            let points = span?.size ?? size
            if let face = face(family: span?.family ?? family, weight: span?.weight ?? weight,
                               italic: span?.italic ?? italic) {
                let font = CTFontCreateWithName(face.name as CFString, points, nil)
                return CTFontGetAscent(font) + CTFontGetDescent(font) + CTFontGetLeading(font)
            }
            #if os(iOS)
            return UIFont.systemFont(ofSize: points).lineHeight
            #else
            let font = NSFont.systemFont(ofSize: points)
            return font.ascender - font.descender + font.leading
            #endif
        }
        return spans.lazy.filter { !$0.value.isEmpty }.map { height($0) }.max() ?? height(nil)
    }
}

private extension WeftWeight {
    var assetWeight: Int {
        switch self {
        case .light: 300
        case .regular: 400
        case .medium: 500
        case .semibold: 600
        case .bold: 700
        }
    }

    var swiftUI: Font.Weight {
        switch self {
        case .light: .light
        case .regular: .regular
        case .medium: .medium
        case .semibold: .semibold
        case .bold: .bold
        }
    }
}
