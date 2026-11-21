import SwiftUI
#if os(iOS)
import UIKit
#else
import AppKit
#endif

struct WeftImageContent {
    let image: Image
    let size: CGSize
}

extension WeftImageContent {
    init?(data: Data) {
        #if os(iOS)
        guard let native = UIImage(data: data) else { return nil }
        self.init(image: Image(uiImage: native), size: native.size)
        #else
        guard let native = NSImage(data: data), native.isValid else { return nil }
        if let bitmap = native.representations.first(where: { $0 is NSBitmapImageRep }) {
            // Network bitmaps use their pixel dimensions, not the file's print DPI.
            native.size = CGSize(width: bitmap.pixelsWide, height: bitmap.pixelsHigh)
        }
        self.init(image: Image(nsImage: native), size: native.size)
        #endif
    }
}

/// Displays a bundled or remote image, measuring its box before applying its fit.
public struct WeftImage: View {
    private let src: String
    private let fit: WeftImageFit
    private let placeholder: WeftColor?
    @Environment(\.weftAssets) private var assets
    @Environment(\.displayScale) private var displayScale
    @State private var remote: (src: String, content: WeftImageContent)?

    public init(src: String, fit: WeftImageFit = .cover, placeholder: WeftColor? = nil) {
        self.src = src
        self.fit = fit
        self.placeholder = placeholder
    }

    public var body: some View {
        if src.hasPrefix("asset:") {
            let name = String(src.dropFirst("asset:".count))
            let asset = assets?.image(name, scale: displayScale)
            #if DEBUG
            box(asset).onChange(of: name, initial: true) {
                if asset == nil { logger.warning("weft: missing image asset \(name, privacy: .public)") }
            }
            #else
            box(asset)
            #endif
        } else {
            box(remote?.src == src ? remote?.content : nil)
                .task(id: src) {
                    remote = nil
                    guard let url = URL(string: src),
                          let (data, response) = try? await URLSession.shared.data(from: url),
                          !Task.isCancelled,
                          (response as? HTTPURLResponse).map({ (200..<300).contains($0.statusCode) }) != false,
                          let content = WeftImageContent(data: data) else { return }
                    remote = (src, content)
                }
        }
    }

    private func box(_ content: WeftImageContent?) -> some View {
        weftImageBox(intrinsic: content?.size ?? .zero) {
            if let content, content.size.width > 0, content.size.height > 0 {
                Color.clear.overlay {
                    let image = content.image.resizable()
                    switch fit {
                    case .cover: image.aspectRatio(content.size.width / content.size.height, contentMode: .fill)
                    case .contain: image.aspectRatio(content.size.width / content.size.height, contentMode: .fit)
                    case .fill: image
                    }
                }
                .clipped()
            } else {
                placeholder?.resolved ?? .clear
            }
        }
    }
}
