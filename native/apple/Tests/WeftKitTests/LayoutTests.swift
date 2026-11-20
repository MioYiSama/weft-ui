import Foundation
import Testing
@testable import WeftKit

private struct LayoutCorpus: Decodable {
    let cases: [Case]

    struct Case: Decodable {
        enum Kind: String, Decodable { case linear, stack }

        let name: String
        let kind: Kind
        let axis: String
        let gap: CGFloat?
        let align: String?
        let justify: String?
        let alignment: String?
        let padding: [CGFloat]
        let proposal: [Length?]
        let children: [Child]
        let size: [CGFloat]
        let frames: [[CGFloat]]
    }

    struct Length: Decodable {
        let exact: CGFloat?
        let atMost: CGFloat?

        var value: WeftProposedLength {
            if let exact { return .exact(exact) }
            if let atMost { return .atMost(atMost) }
            return .unbounded
        }
    }

    struct Child: Decodable {
        let intrinsic: [CGFloat]
        let width: CGFloat?
        let height: CGFloat?
        let grow: CGFloat?
        let fill: Bool?
        let margin: [CGFloat]?
        let aspectRatio: CGFloat?
        let spacer: Bool?
        let size: CGFloat?

        var layout: WeftLayoutChild {
            WeftLayoutChild(width: width, height: height, grow: grow ?? 0, fill: fill ?? false,
                            margin: margin.map(edges) ?? WeftInsets(),
                            spacer: spacer == true ? WeftSpacerValue(size: size) : nil)
        }

        func measure(_ proposal: WeftProposal) -> CGSize {
            WeftSizeConstraints(width: width, height: height, aspectRatio: aspectRatio)
                .measure(proposal, padding: WeftInsets()) { _ in
                    CGSize(width: intrinsic[0], height: intrinsic[1])
                }
        }
    }
}

private func edges(_ values: [CGFloat]) -> WeftInsets {
    WeftInsets(top: values[0], right: values[1], bottom: values[2], left: values[3])
}

@Test func measuresLayoutCorpus() throws {
    let corpus = try JSONDecoder().decode(LayoutCorpus.self, from: corpusData("layout"))
    for test in corpus.cases {
        let axis = try #require(WeftAxis(rawValue: test.axis))
        let proposal = WeftProposal(axis: axis, main: test.proposal[0]?.value ?? .unbounded,
                                    cross: test.proposal[1]?.value ?? .unbounded)
        let padding = edges(test.padding)
        let children = test.children.map(\.layout)
        func measure(_ index: Int, _ proposal: WeftProposal) -> CGSize {
            test.children[index].measure(proposal)
        }
        let result: WeftLayoutResult
        switch test.kind {
        case .linear:
            let align = try #require(test.align.flatMap(WeftCrossAlign.init(rawValue:)))
            let justify = try #require(test.justify.flatMap(WeftJustify.init(rawValue:)))
            result = WeftLayoutAlgorithm.linear(axis: axis, gap: test.gap ?? 0, align: align, justify: justify,
                                                padding: padding, proposal: proposal, children: children, measure: measure)
        case .stack:
            let alignment = try #require(test.alignment.flatMap(WeftAlignment.init(rawValue:)))
            result = WeftLayoutAlgorithm.stack(alignment: alignment, padding: padding, proposal: proposal,
                                               children: children, measure: measure)
        }
        #expect(abs(result.size.width - test.size[0]) <= 0.001, "\(test.name) width")
        #expect(abs(result.size.height - test.size[1]) <= 0.001, "\(test.name) height")
        #expect(result.placements.count == test.frames.count, "\(test.name) frame count")
        for (i, placement) in result.placements.enumerated() {
            let frame = placement.frame
            let expected = test.frames[i]
            #expect(abs(frame.minX - expected[0]) <= 0.001, "\(test.name) child \(i) x")
            #expect(abs(frame.minY - expected[1]) <= 0.001, "\(test.name) child \(i) y")
            #expect(abs(frame.width - expected[2]) <= 0.001, "\(test.name) child \(i) width")
            #expect(abs(frame.height - expected[3]) <= 0.001, "\(test.name) child \(i) height")
        }
    }
}

@Test func fixedAndMinMaxSizesConstrainTheContentBoxBeforePadding() {
    let padding = WeftInsets(padding: 2, paddingX: 6, paddingTop: 4, paddingEnd: 8)
    let constraints = WeftSizeConstraints(width: 70, maxWidth: 50, minHeight: 12)
    let size = constraints.measure(WeftProposal(width: .exact(40), height: .atMost(80)), padding: padding) { proposal in
        #expect(proposal.width == .exact(50))
        #expect(proposal.height == .atMost(74))
        return CGSize(width: 3, height: 4)
    }
    #expect(size == CGSize(width: 64, height: 18))
}
