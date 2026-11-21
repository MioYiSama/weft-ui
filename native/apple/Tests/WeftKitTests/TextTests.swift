import AppKit
import SwiftUI
import Testing
@testable import WeftKit

@MainActor @Test func joinsSpanValuesAndInheritsOnlyUnsetStyles() {
    let inherited = WeftColor.rgba(0x336655ff)
    let override = WeftColor.rgba(0x123456ff)
    let text = WeftText(spans: [
        WeftSpan(value: " a "),
        WeftSpan(value: "", size: 30),
        WeftSpan(value: "🙂\n", color: override, size: 14, weight: .bold, italic: false,
                 decoration: WeftDecoration.none),
        WeftSpan(value: "é ", decoration: .lineThrough),
    ], color: inherited, size: 18, weight: .medium, italic: true, decoration: .underline).attributedText

    #expect(String(text.characters) == " a 🙂\né ")
    let runs = Array(text.runs)
    guard runs.count == 3 else {
        Issue.record("Expected one attributed run per nonempty span: \(runs.count)")
        return
    }
    let inheritedFont = Font.system(size: 18).weight(.medium).italic()
    #expect(runs[0].swiftUI.font == inheritedFont)
    #expect(runs[0].swiftUI.foregroundColor == inherited.resolved)
    #expect(runs[0].swiftUI.underlineStyle == .single)
    #expect(runs[1].swiftUI.font == Font.system(size: 14).weight(.bold))
    #expect(runs[1].swiftUI.foregroundColor == override.resolved)
    #expect(runs[1].swiftUI.underlineStyle == nil)
    #expect(runs[1].swiftUI.strikethroughStyle == nil)
    #expect(runs[2].swiftUI.font == inheritedFont)
    #expect(runs[2].swiftUI.underlineStyle == nil)
    #expect(runs[2].swiftUI.strikethroughStyle == .single)
}

@MainActor @Test func textWrapsByDefaultAndOnlyPositiveLineLimitsTruncate() {
    let spans = [WeftSpan(value: "A paragraph that wraps into several lines at this narrow width.")]
    func height(maxLines: CGFloat) -> CGFloat {
        NSHostingView(rootView: weftText(spans: spans, size: 15, maxLines: maxLines)
            .frame(width: 90)).fittingSize.height
    }
    let one = height(maxLines: 1)
    let two = height(maxLines: 2)
    let unlimited = height(maxLines: 0)
    #expect(one > 0)
    #expect(two > one)
    #expect(unlimited > two)
    #expect(height(maxLines: -2) == unlimited)
}

@MainActor @Test func textOnlyAddsLineSpacingAboveTheFontHeight() {
    func height(lineHeight: CGFloat?) -> CGFloat {
        NSHostingView(rootView: weftText(spans: [WeftSpan(value: "first\nsecond")],
                                        size: 15, lineHeight: lineHeight)).fittingSize.height
    }
    let natural = height(lineHeight: nil)
    #expect(natural > 0)
    #expect(height(lineHeight: 10) == natural)
    #expect(height(lineHeight: 36) > height(lineHeight: 24))
    #expect(height(lineHeight: 24) > natural)
}
