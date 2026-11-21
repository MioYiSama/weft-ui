import Foundation

/// The native layout protocol uses typed proposals independent of SwiftUI's layout API.
enum WeftProposedLength: Equatable {
    case exact(CGFloat), atMost(CGFloat), unbounded

    var limit: CGFloat? {
        switch self {
        case let .exact(value), let .atMost(value): value
        case .unbounded: nil
        }
    }

    var exact: CGFloat? {
        if case let .exact(value) = self { return value }
        return nil
    }

    func inset(_ amount: CGFloat) -> Self {
        switch self {
        case let .exact(value): .exact(max(0, value - amount))
        case let .atMost(value): .atMost(max(0, value - amount))
        case .unbounded: .unbounded
        }
    }

    func resolve(_ ideal: CGFloat) -> CGFloat {
        switch self {
        case let .exact(value): value
        case let .atMost(value): min(ideal, value)
        case .unbounded: ideal
        }
    }
}

struct WeftProposal: Equatable {
    var width: WeftProposedLength = .unbounded
    var height: WeftProposedLength = .unbounded

    init(width: WeftProposedLength = .unbounded, height: WeftProposedLength = .unbounded) {
        self.width = width
        self.height = height
    }

    init(axis: WeftAxis, main: WeftProposedLength, cross: WeftProposedLength) {
        (width, height) = axis == .horizontal ? (main, cross) : (cross, main)
    }

    func imageIdeal(intrinsic: CGSize) -> CGSize {
        guard intrinsic.width > 0, intrinsic.height > 0 else { return .zero }
        let w = intrinsic.width
        let h = intrinsic.height
        if let width = width.exact { return CGSize(width: width, height: width * h / w) }
        if let height = height.exact { return CGSize(width: height * w / h, height: height) }
        let scale = min(1, (width.limit ?? .infinity) / w, (height.limit ?? .infinity) / h)
        return CGSize(width: w * scale, height: h * scale)
    }

    func inset(_ edges: WeftInsets) -> Self {
        Self(width: width.inset(edges.horizontal), height: height.inset(edges.vertical))
    }
}

/// Four physical edges of a padding or margin box, in logical points.
public struct WeftInsets: Equatable, Sendable {
    var top: CGFloat
    var right: CGFloat
    var bottom: CGFloat
    var left: CGFloat

    init(top: CGFloat, right: CGFloat, bottom: CGFloat, left: CGFloat) {
        self.top = top
        self.right = right
        self.bottom = bottom
        self.left = left
    }

    public init(
        padding: CGFloat? = nil, paddingX: CGFloat? = nil, paddingY: CGFloat? = nil,
        paddingTop: CGFloat? = nil, paddingBottom: CGFloat? = nil,
        paddingStart: CGFloat? = nil, paddingEnd: CGFloat? = nil
    ) {
        self.init(
            top: max(0, paddingTop ?? paddingY ?? padding ?? 0),
            right: max(0, paddingEnd ?? paddingX ?? padding ?? 0),
            bottom: max(0, paddingBottom ?? paddingY ?? padding ?? 0),
            left: max(0, paddingStart ?? paddingX ?? padding ?? 0))
    }

    var horizontal: CGFloat { left + right }
    var vertical: CGFloat { top + bottom }
}

struct WeftSizeConstraints: Equatable, Sendable {
    var width: CGFloat?
    var height: CGFloat?
    var minWidth: CGFloat?
    var maxWidth: CGFloat?
    var minHeight: CGFloat?
    var maxHeight: CGFloat?
    var aspectRatio: CGFloat?

    private func clamp(_ value: CGFloat, min low: CGFloat?, max high: CGFloat?) -> CGFloat {
        max(0, low ?? 0, min(value, high ?? .infinity))
    }

    func clamp(_ size: CGSize) -> CGSize {
        CGSize(width: clamp(size.width, min: minWidth, max: maxWidth),
               height: clamp(size.height, min: minHeight, max: maxHeight))
    }

    func contentProposal(_ outer: WeftProposal, padding: WeftInsets) -> WeftProposal {
        let proposal = outer.inset(padding)
        var w = width ?? proposal.width.exact
        var h = height ?? proposal.height.exact
        if let ratio = aspectRatio, ratio.isFinite, ratio > 0 {
            if let w, height == nil {
                h = w / ratio
            } else if let h, width == nil {
                w = h * ratio
            }
        }
        func limit(
            _ exact: CGFloat?, _ proposed: WeftProposedLength, min low: CGFloat?, max high: CGFloat?
        ) -> WeftProposedLength {
            if let exact { return .exact(clamp(exact, min: low, max: high)) }
            if let maximum = proposed.limit ?? high {
                return .atMost(clamp(maximum, min: low, max: high))
            }
            return .unbounded
        }
        return WeftProposal(
            width: limit(w, proposal.width, min: minWidth, max: maxWidth),
            height: limit(h, proposal.height, min: minHeight, max: maxHeight))
    }

    func measure(
        _ outer: WeftProposal, padding: WeftInsets, intrinsic: (WeftProposal) -> CGSize
    ) -> CGSize {
        let proposal = contentProposal(outer, padding: padding)
        let ideal = intrinsic(proposal)
        let size = clamp(CGSize(width: proposal.width.resolve(ideal.width), height: proposal.height.resolve(ideal.height)))
        return CGSize(width: size.width + padding.horizontal, height: size.height + padding.vertical)
    }
}

struct WeftLayoutChild {
    var width: CGFloat?
    var height: CGFloat?
    var grow: CGFloat = 0
    var fill = false
    var margin = WeftInsets()
    var spacer: WeftSpacerValue?

    var weight: CGFloat {
        if let spacer { return spacer.size == nil ? 1 : 0 }
        return grow.isFinite ? max(0, grow) : 0
    }
}

struct WeftSpacerValue: Sendable {
    var size: CGFloat?
}

struct WeftPlacement {
    var frame = CGRect.zero
    var proposal = WeftProposal()
}

struct WeftLayoutResult {
    let size: CGSize
    let placements: [WeftPlacement]
}

extension WeftAxis {
    func main(_ size: CGSize) -> CGFloat { self == .horizontal ? size.width : size.height }
    func cross(_ size: CGSize) -> CGFloat { self == .horizontal ? size.height : size.width }
    func main(_ proposal: WeftProposal) -> WeftProposedLength { self == .horizontal ? proposal.width : proposal.height }
    func cross(_ proposal: WeftProposal) -> WeftProposedLength { self == .horizontal ? proposal.height : proposal.width }
    func before(_ edges: WeftInsets) -> CGFloat { self == .horizontal ? edges.left : edges.top }
    func start(_ edges: WeftInsets) -> CGFloat { self == .horizontal ? edges.top : edges.left }
    func along(_ edges: WeftInsets) -> CGFloat { self == .horizontal ? edges.horizontal : edges.vertical }
    func across(_ edges: WeftInsets) -> CGFloat { self == .horizontal ? edges.vertical : edges.horizontal }
    func size(main: CGFloat, cross: CGFloat) -> CGSize {
        self == .horizontal ? CGSize(width: main, height: cross) : CGSize(width: cross, height: main)
    }
    func origin(main: CGFloat, cross: CGFloat) -> CGPoint {
        self == .horizontal ? CGPoint(x: main, y: cross) : CGPoint(x: cross, y: main)
    }
}

/// The shared layout algorithm. `measure` sees only visible children and returns border-box sizes.
enum WeftLayoutAlgorithm {
    static func linear(
        axis: WeftAxis, gap: CGFloat, align: WeftCrossAlign, justify: WeftJustify,
        padding: WeftInsets, proposal: WeftProposal, children: [WeftLayoutChild],
        measure: (Int, WeftProposal) -> CGSize
    ) -> WeftLayoutResult {
        let proposal = proposal.inset(padding)
        let (main, cross) = (axis.main(proposal), axis.cross(proposal))
        let gap = max(0, gap)
        var used = gap * CGFloat(max(0, children.count - 1))
        var crossUsed: CGFloat = 0
        var weight: CGFloat = 0
        var lastGrowing: Int?
        var placements = Array(repeating: WeftPlacement(), count: children.count)
        func crossProposal(_ child: WeftLayoutChild) -> WeftProposedLength {
            guard let limit = cross.limit else { return .unbounded }
            let available = max(0, limit - axis.across(child.margin))
            let fixed = axis == .horizontal ? child.height : child.width
            return child.spacer == nil && fixed == nil && (align == .stretch || child.fill)
                ? .exact(available) : .atMost(available)
        }
        func measureChild(_ index: Int, _ child: WeftLayoutChild, _ length: WeftProposedLength) {
            let size: CGSize
            let proposed: WeftProposal
            if let spacer = child.spacer {
                let mainSize = max(0, spacer.size ?? length.exact ?? 0)
                size = axis.size(main: mainSize, cross: 0)
                proposed = WeftProposal(axis: axis, main: .exact(mainSize), cross: .exact(0))
            } else {
                proposed = WeftProposal(axis: axis, main: length, cross: crossProposal(child))
                size = measure(index, proposed)
            }
            placements[index] = WeftPlacement(frame: CGRect(origin: .zero, size: size), proposal: proposed)
            used += axis.main(size)
            crossUsed = max(crossUsed, axis.cross(size) + axis.across(child.margin))
        }
        for (i, child) in children.enumerated() {
            used += axis.along(child.margin)
            if main.limit != nil, child.weight > 0 {
                weight += child.weight
                lastGrowing = i
            } else {
                measureChild(i, child, .unbounded)
            }
        }
        if let available = main.limit, let lastGrowing {
            let free = max(0, available - used)
            var remaining = free
            for (i, child) in children.enumerated() where child.weight > 0 {
                let share = i == lastGrowing ? remaining : min(remaining, free * (child.weight / weight))
                remaining -= share
                measureChild(i, child, .exact(share))
            }
        }
        let mainSize = main.resolve(used)
        let crossSize = cross.resolve(crossUsed)
        let remainder = mainSize - used
        let count = CGFloat(children.count)
        let offset: CGFloat
        let extraGap: CGFloat
        switch justify {
        case .start: (offset, extraGap) = (0, 0)
        case .center: (offset, extraGap) = (remainder / 2, 0)
        case .end: (offset, extraGap) = (remainder, 0)
        case .spaceBetween: (offset, extraGap) = (0, count > 1 ? remainder / (count - 1) : 0)
        case .spaceAround: (offset, extraGap) = count > 0 ? (remainder / (2 * count), remainder / count) : (0, 0)
        case .spaceEvenly:
            let space = remainder / (count + 1)
            (offset, extraGap) = (space, space)
        }
        var cursor = axis.before(padding) + offset
        for (i, child) in children.enumerated() {
            let extra = crossSize - axis.cross(placements[i].frame.size) - axis.across(child.margin)
            let crossOffset: CGFloat
            if crossProposal(child).exact != nil {
                crossOffset = 0
            } else {
                switch align {
                case .start, .stretch: crossOffset = 0
                case .center: crossOffset = extra / 2
                case .end: crossOffset = extra
                }
            }
            cursor += axis.before(child.margin)
            placements[i].frame.origin = axis.origin(
                main: cursor, cross: axis.start(padding) + axis.start(child.margin) + crossOffset)
            cursor += axis.main(placements[i].frame.size) + axis.along(child.margin) - axis.before(child.margin) + gap + extraGap
        }
        return WeftLayoutResult(
            size: axis.size(main: mainSize + axis.along(padding), cross: crossSize + axis.across(padding)),
            placements: placements)
    }

    static func stack(
        alignment: WeftAlignment, padding: WeftInsets, proposal: WeftProposal,
        children: [WeftLayoutChild], measure: (Int, WeftProposal) -> CGSize
    ) -> WeftLayoutResult {
        let proposal = proposal.inset(padding)
        var ideal = CGSize.zero
        var placements = Array(repeating: WeftPlacement(), count: children.count)
        for (i, child) in children.enumerated() where !child.fill {
            let available = proposal.inset(child.margin)
            let proposed = WeftProposal(
                width: available.width.limit.map(WeftProposedLength.atMost) ?? .unbounded,
                height: available.height.limit.map(WeftProposedLength.atMost) ?? .unbounded)
            let size = measure(i, proposed)
            placements[i] = WeftPlacement(frame: CGRect(origin: .zero, size: size), proposal: proposed)
            ideal.width = max(ideal.width, size.width + child.margin.horizontal)
            ideal.height = max(ideal.height, size.height + child.margin.vertical)
        }
        let width = proposal.width.resolve(ideal.width)
        let height = proposal.height.resolve(ideal.height)
        let content = WeftProposal(width: .exact(width), height: .exact(height))
        let anchor: (x: CGFloat, y: CGFloat)
        switch alignment {
        case .topStart: anchor = (0, 0)
        case .top: anchor = (0.5, 0)
        case .topEnd: anchor = (1, 0)
        case .start: anchor = (0, 0.5)
        case .center: anchor = (0.5, 0.5)
        case .end: anchor = (1, 0.5)
        case .bottomStart: anchor = (0, 1)
        case .bottom: anchor = (0.5, 1)
        case .bottomEnd: anchor = (1, 1)
        }
        for (i, child) in children.enumerated() {
            if child.fill {
                let proposed = content.inset(child.margin)
                placements[i] = WeftPlacement(frame: CGRect(origin: .zero, size: measure(i, proposed)), proposal: proposed)
            }
            let size = placements[i].frame.size
            placements[i].frame.origin = CGPoint(
                x: padding.left + child.margin.left + anchor.x * (width - child.margin.horizontal - size.width),
                y: padding.top + child.margin.top + anchor.y * (height - child.margin.vertical - size.height))
        }
        return WeftLayoutResult(
            size: CGSize(width: width + padding.horizontal, height: height + padding.vertical), placements: placements)
    }
}
