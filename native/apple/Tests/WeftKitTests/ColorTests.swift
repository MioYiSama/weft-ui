import SwiftUI
import Testing
@testable import WeftKit

@Test func parsesColorsFromValueSlots() {
    #expect(WeftValue.string("#AbC").color == .rgba(0xaabbccff))
    #expect(WeftValue.string("#123456").color == .rgba(0x123456ff))
    #expect(WeftValue.string("#12345680").color == .rgba(0x12345680))
    #expect(WeftValue.string("@_accent2").color == .token("_accent2"))
    for text in ["", "@", "@1accent", "@accent-color", "#ff", "#-00", "#ébc", "#123456789"] {
        #expect(WeftValue.string(text).color == .clear, "Invalid color: \(text)")
    }
}

@MainActor @Test func themeColorsResolveForEachAppearance() {
    WeftTheme.register([
        "accent": (0xff0000ff, 0x0000ffff),
        "invariant": (0x12345680, 0x12345680),
    ])
    defer { WeftTheme.register([:]) }
    let accent = WeftColor.token("accent").resolved
    let invariant = WeftColor.token("invariant").resolved
    let unknown = WeftColor.token("missing").resolved
    var environment = EnvironmentValues()
    environment.colorScheme = .light
    #expect(accent.resolve(in: environment) == WeftColor.rgba(0xff0000ff).resolved.resolve(in: environment))
    environment.colorScheme = .dark
    #expect(accent.resolve(in: environment) == WeftColor.rgba(0x0000ffff).resolved.resolve(in: environment))
    #expect(invariant.resolve(in: environment) == WeftColor.rgba(0x12345680).resolved.resolve(in: environment))
    #expect(unknown.resolve(in: environment) == Color.clear.resolve(in: environment))
}
