// swift-tools-version: 6.0
// WeftKit: the Apple platform shell of the Weft native kernel. It runs the
// kernel (binary target WeftCore, built by build-core.sh), applies its view
// protocol ops to the scopes of the SwiftUI views that weft_ui_transformer::swiftui
// generates, and implements the platform modules.

import PackageDescription

let package = Package(
    name: "WeftKit",
    // Keep in sync with build-core.sh.
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "WeftKit", targets: ["WeftKit"]),
    ],
    targets: [
        .binaryTarget(name: "WeftCore", path: "build/WeftCore.xcframework"),
        .target(name: "WeftKit", dependencies: ["WeftCore"]),
        .testTarget(name: "WeftKitTests", dependencies: ["WeftKit"]),
    ]
)
