import Foundation
#if os(iOS)
import UIKit
#elseif os(macOS)
import AppKit
#endif

struct WeftEnvironment: Encodable, Equatable {
    struct Screen: Encodable, Equatable {
        let width: CGFloat
        let height: CGFloat
        let scale: CGFloat
    }

    struct Insets: Encodable, Equatable {
        var top: CGFloat = 0
        var bottom: CGFloat = 0
        var start: CGFloat = 0
        var end: CGFloat = 0
    }

    let colorScheme: String
    let screen: Screen
    let safeArea: Insets
    let orientation: String

    init(dark: Bool, size: CGSize, scale: CGFloat, insets: Insets) {
        colorScheme = dark ? "dark" : "light"
        screen = Screen(width: size.width, height: size.height, scale: scale)
        safeArea = insets
        orientation = size.width > size.height ? "landscape" : "portrait"
    }

    @MainActor static func initial() -> WeftEnvironment {
        #if os(iOS)
        // This may run while SwiftUI initializes its view tree. Reading
        // UIWindow.safeAreaInsets here would re-enter SwiftUI via the status bar.
        let screen = UIScreen.main
        let style = UITraitCollection.current.userInterfaceStyle
        let dark = (style == .unspecified ? screen.traitCollection.userInterfaceStyle : style) == .dark
        let size = screen.bounds.size
        let scale = screen.scale
        let insets = Insets()
        #elseif os(macOS)
        let screen = NSScreen.main
        let size = screen?.frame.size ?? .zero
        let scale = screen?.backingScaleFactor ?? 1
        let dark = NSApp?.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let insets = Insets()
        #endif
        return WeftEnvironment(dark: dark, size: size, scale: scale, insets: insets)
    }

    func json() -> Data {
        // All numbers come from finite window geometry and display scale.
        let encoder = JSONEncoder()
        encoder.outputFormatting = .sortedKeys
        return try! encoder.encode(self)
    }
}
