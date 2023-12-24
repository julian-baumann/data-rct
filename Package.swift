// swift-tools-version: 5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "DataRCT",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "DataRCT",
            targets: ["DataRCT"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "DataRCT",
            dependencies: ["DataRCTFFI"],
            path: "bindings/swift/Sources"
        ),

        .binaryTarget(
            name: "DataRCTFFI",
            path: "bindings/swift/DataRCTFFI.xcframework"
        ),

        .testTarget(
            name: "DataRCTTests",
            dependencies: ["DataRCT"],
            path: "bindings/swift/Tests"
        ),
    ]
)
