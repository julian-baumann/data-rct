// swift-tools-version: 5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let rustBuildDir = "../../data_rct_ffi/target/debug/"

let package = Package(
    name: "DataRCT",
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "DataRCT",
            targets: ["DataRCT"]),
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        // .package(url: /* package url */, from: "1.0.0"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "DataRCT",
            dependencies: []),
        
        .binaryTarget(
            name: "DataRCT_FFI",
            url: "https://github.com/julian-baumann/data-rct/blob/feature/swift-ffi/ffi/swift/DataRCT.xcframework.zip?raw=true",
            checksum: "6258a6c8edefec07f56e4c3399799f0bb5fd7714b25e6044d6d80cca0605252a"),
        
        .testTarget(
            name: "DataRCTTests",
            dependencies: ["DataRCT"]),
    ]
)
