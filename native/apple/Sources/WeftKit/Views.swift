import SwiftUI

/// Renders the root component of a session, or why the app failed. The page
/// scrolls vertically and its content starts at the top leading corner.
public struct WeftRoot: View {
    private let session: WeftSession

    public init(session: WeftSession) {
        self.session = session
    }

    public var body: some View {
        ScrollView(.vertical) {
            Group {
                if let failure = session.failure {
                    Text(verbatim: failure)
                        .foregroundStyle(.red)
                        .padding()
                } else if let root = session.root {
                    root.view.id(root.id)
                }
            }
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
    }
}

extension View {
    /// Layout properties of `Column` and `Row`: a fixed content size, then
    /// padding around it, then the background behind both, as CSS boxes do.
    public func weftBox(padding: CGFloat?, width: CGFloat?, height: CGFloat?, background: Color?) -> some View {
        frame(width: width, height: height, alignment: .topLeading)
            .padding(padding ?? 0)
            .background(background ?? .clear)
    }

    /// Style properties of `Text`; unset ones keep the platform's defaults.
    public func weftText(color: Color?, size: CGFloat?, bold: Bool?) -> some View {
        foregroundStyle(color ?? .primary)
            .font(size.map { Font.system(size: $0) })
            .bold(bold ?? false)
    }
}

/// `Image`: a remote image. With one fixed side the other follows the
/// aspect ratio, with both the image fills them, and with neither it keeps
/// its own size.
public struct WeftImage: View {
    private let src: String
    private let width: CGFloat?
    private let height: CGFloat?

    public init(src: String, width: CGFloat?, height: CGFloat?) {
        self.src = src
        self.width = width
        self.height = height
    }

    public var body: some View {
        AsyncImage(url: URL(string: src)) { phase in
            if let image = phase.image {
                switch (width, height) {
                case (nil, nil): image
                case (.some, .some): image.resizable()
                default: image.resizable().aspectRatio(contentMode: .fit)
                }
            } else {
                Color.clear.frame(width: width ?? 0, height: height ?? 0)
            }
        }
        .frame(width: width, height: height)
    }
}
