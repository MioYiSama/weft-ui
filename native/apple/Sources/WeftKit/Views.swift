import SwiftUI

/// Renders the root component or startup failure in the window's safe area.
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
            weftLinear(axis: .vertical) {
                if let failure = session.failure {
                    Text(verbatim: failure)
                        .foregroundStyle(.red)
                        .padding()
                } else if let root = session.root {
                    root.view.id(root.id)
                }
            }
            .weftFrame(width: geometry.size.width, height: geometry.size.height)
            .clipped()
            .environment(\.layoutDirection, .leftToRight)
            .onChange(of: environment, initial: true) { _, updated in
                session.update(environment: updated)
            }
        }
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

extension View {
    /// Style properties of `Text`; unset ones keep the platform's defaults.
    public func weftText(color: WeftColor?, size: CGFloat?, bold: Bool?) -> some View {
        foregroundStyle(color?.resolved ?? .primary)
            .font(size.map { Font.system(size: $0) })
            .bold(bold ?? false)
    }
}

/// `Image`: a remote image. With one fixed side the other follows the
/// aspect ratio, with both the image fills them, and with neither it keeps
/// its own size.
public struct WeftImage: View {
    private let src: String
    private var sizing = WeftSizing()

    public init(src: String) { self.src = src }

    public var body: some View {
        let axes = sizing.exactAxes
        let width = sizing.frame.width != nil || axes.width
        let height = sizing.frame.height != nil || axes.height
        weftLeaf {
            AsyncImage(url: URL(string: src)) { phase in
                if let image = phase.image {
                    switch (width, height) {
                    case (false, false): image
                    case (true, true): image.resizable()
                    default: image.resizable().aspectRatio(contentMode: .fit)
                    }
                }
            }
        }
    }
}
