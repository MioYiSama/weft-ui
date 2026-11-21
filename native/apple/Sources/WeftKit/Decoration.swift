import SwiftUI

extension View {
    /// Changes the opacity of the complete box, including its children and decoration.
    @MainActor public func weftOpacity(opacity: CGFloat) -> some View {
        self.opacity(min(1, max(0, opacity)))
    }

    /// Draws decoration around the padding box without changing content-box constraints.
    @MainActor public func weftDecoration(
        background: WeftColor? = nil, radius: CGFloat = 0, border: CGFloat = 0,
        borderColor: WeftColor? = nil, shadow: WeftShadow = .none, clip: Bool = false
    ) -> some View {
        modifier(WeftBoxDecoration(background: background, radius: max(0, radius), border: max(0, border),
                                   borderColor: borderColor, shadow: shadow, clip: clip))
    }
}

private struct WeftBoxDecoration: ViewModifier {
    @Environment(\.colorScheme) private var colorScheme
    let background: WeftColor?
    let radius: CGFloat
    let border: CGFloat
    let borderColor: WeftColor?
    let shadow: WeftShadow
    let clip: Bool

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: radius, style: .circular)
        Group {
            if clip {
                content.clipShape(WeftPaddingShape(radius: radius, border: border))
            } else {
                content
            }
        }
        .overlay {
            if border > 0 {
                let color = borderColor ?? .rgba(colorScheme == .dark ? 0xffffff26 : 0x0000001f)
                shape.strokeBorder(color.resolved, lineWidth: border)
                    .allowsHitTesting(false)
            }
        }
        .background {
            if let background {
                shape.fill(background.resolved).allowsHitTesting(false)
            }
        }
        .background {
            if let shadow = shadow.parameters {
                // Subtract the caster so even a transparent box only paints a shadow outside its bounds.
                shape.fill(.black)
                    .shadow(color: WeftColor.rgba(shadow.rgba).resolved, radius: shadow.blur / 2, x: 0, y: shadow.y)
                    .overlay { shape.fill(.black).blendMode(.destinationOut) }
                    .compositingGroup()
                    .allowsHitTesting(false)
            }
        }
        .environment(\.weftBorder, border)
    }
}

struct WeftPaddingShape: Shape {
    let radius: CGFloat
    let border: CGFloat

    func path(in rect: CGRect) -> Path {
        guard rect.width > 2 * border, rect.height > 2 * border else { return Path() }
        let innerRadius = max(0, min(radius, min(rect.width, rect.height) / 2) - border)
        return RoundedRectangle(cornerRadius: innerRadius, style: .circular)
            .path(in: rect.insetBy(dx: border, dy: border))
    }
}

extension WeftShadow {
    var parameters: (y: CGFloat, blur: CGFloat, rgba: UInt32)? {
        switch self {
        case .none: nil
        case .small: (1, 3, 0x0000001f)
        case .medium: (4, 12, 0x00000026)
        case .large: (12, 32, 0x0000002e)
        }
    }
}
