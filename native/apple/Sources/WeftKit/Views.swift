import SwiftUI

/// Renders the root component or startup failure, respecting the safe area by default.
public struct WeftRoot: View {
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.displayScale) private var displayScale
    private let session: WeftSession

    public init(session: WeftSession) {
        self.session = session
    }

    public var body: some View {
        GeometryReader { geometry in
            let environment = windowEnvironment(geometry)
            Group {
                if session.safeArea {
                    content(size: geometry.size)
                } else {
                    // Keep the outer reader in the safe area so platform.safeArea still reports the insets.
                    GeometryReader { fullWindow in
                        content(size: fullWindow.size)
                    }
                    .ignoresSafeArea()
                }
            }
            .onChange(of: environment, initial: true) { _, updated in
                session.update(environment: updated)
            }
        }
    }

    private func content(size: CGSize) -> some View {
        weftLinear(axis: .vertical) {
            if let failure = session.failure {
                Text(verbatim: failure)
                    .foregroundStyle(.red)
                    .padding()
            } else if let root = session.root {
                root.view.id(root.id)
            }
        }
        .weftFrame(width: size.width, height: size.height)
        .clipped()
        .environment(\.layoutDirection, .leftToRight)
        .environment(\.weftAssets, session.assets)
    }

    private func windowEnvironment(_ geometry: GeometryProxy) -> WeftEnvironment {
        #if os(iOS)
        let safe = geometry.safeAreaInsets
        let insets = WeftEnvironment.Insets(top: safe.top, bottom: safe.bottom, start: safe.leading, end: safe.trailing)
        #else
        let insets = WeftEnvironment.Insets()
        #endif
        // SwiftUI lays out this reader inside the safe area; include the insets in the full size.
        let size = CGSize(
            width: geometry.size.width + insets.start + insets.end,
            height: geometry.size.height + insets.top + insets.bottom)
        return WeftEnvironment(dark: colorScheme == .dark, size: size, scale: displayScale, insets: insets)
    }
}

/// Gives tappable Weft elements native button behavior without changing their layout or colors.
public struct WeftPressStyle: ButtonStyle {
    public init() {}

    public func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .contentShape(Rectangle())
            .opacity(configuration.isPressed ? 0.6 : 1)
    }
}

/// Keeps interactive descendants of a button label individually accessible.
@MainActor public func weftButtonWithChildren<Label: View>(
    action: @escaping () -> Void,
    @ViewBuilder label: () -> Label
) -> some View {
    let content = label()
    let button = Button(action: action) { content }
    #if os(macOS)
    // SwiftUI exposes the button label but drops its interactive descendants on macOS.
    return button.accessibilityChildren { content.buttonStyle(.automatic) }
    #else
    return button
    #endif
}

