import CoreText
import SwiftUI
#if os(iOS)
import UIKit
#else
import AppKit
#endif

extension EnvironmentValues {
    @Entry var weftAssets: WeftAssets?
}

/// The resources of one generated Swift package.
@MainActor
struct WeftAssets {
    let bundle: Bundle
    private let fonts: [String: [FontFace]]

    init(bundle: Bundle) {
        self.bundle = bundle
        guard let directory = bundle.resourceURL?.appendingPathComponent("fonts", isDirectory: true),
              let data = try? Data(contentsOf: directory.appendingPathComponent("manifest.json")),
              let entries = try? JSONDecoder().decode([FontEntry].self, from: data) else {
            fonts = [:]
            return
        }
        var faces: [String: [FontFace]] = [:]
        for entry in entries {
            let url = directory.appendingPathComponent(entry.file) as CFURL
            guard let descriptors = CTFontManagerCreateFontDescriptorsFromURL(url) as? [CTFontDescriptor],
                  let descriptor = descriptors.first,
                  let name = CTFontDescriptorCopyAttribute(descriptor, kCTFontNameAttribute) as? String else {
                logger.warning("weft: could not read font face from \(entry.file, privacy: .public)")
                continue
            }
            var error: Unmanaged<CFError>?
            if !CTFontManagerRegisterFontsForURL(url, .process, &error),
               let error = error?.takeRetainedValue(),
               CFErrorGetCode(error) != CTFontManagerError.alreadyRegistered.rawValue {
                logger.warning("weft: could not register font \(entry.file, privacy: .public)")
                continue
            }
            faces[entry.family, default: []].append(FontFace(name: name, weight: entry.weight, italic: entry.italic))
        }
        fonts = faces
    }

    func image(_ name: String, scale: CGFloat) -> WeftImageContent? {
        #if os(iOS)
        let native = UIImage(named: name, in: bundle, compatibleWith: UITraitCollection(displayScale: scale))
        #else
        let native = bundle.image(forResource: NSImage.Name(name))
        #endif
        guard let native else { return nil }
        return WeftImageContent(image: Image(name, bundle: bundle), size: native.size)
    }

    func face(family: String, weight: Int, italic: Bool) -> FontFace? {
        fonts[family]?.min {
            // Prefer the requested posture, then the closest available weight.
            ($0.italic != italic ? 1 : 0, abs($0.weight - weight))
                < ($1.italic != italic ? 1 : 0, abs($1.weight - weight))
        }
    }

    struct FontFace {
        let name: String
        let weight: Int
        let italic: Bool
    }

    private struct FontEntry: Decodable {
        let file: String
        let family: String
        let weight: Int
        let italic: Bool
    }
}
