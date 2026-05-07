// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ForemanOverlay",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "ForemanOverlayCore", targets: ["ForemanOverlayCore"]),
        .library(name: "ForemanOverlayUI", targets: ["ForemanOverlayUI"]),
        .executable(name: "foreman-overlay", targets: ["ForemanOverlay"]),
        .executable(name: "foreman-overlay-snapshot", targets: ["ForemanOverlaySnapshot"]),
    ],
    dependencies: [
        .package(url: "https://github.com/sindresorhus/KeyboardShortcuts", from: "2.3.0"),
    ],
    targets: [
        .target(name: "ForemanOverlayCore"),
        .target(name: "ForemanOverlayUI", dependencies: ["ForemanOverlayCore", "KeyboardShortcuts"]),
        .executableTarget(name: "ForemanOverlay", dependencies: ["ForemanOverlayCore", "ForemanOverlayUI", "KeyboardShortcuts"]),
        .executableTarget(name: "ForemanOverlaySnapshot", dependencies: ["ForemanOverlayCore", "ForemanOverlayUI"]),
        .testTarget(name: "ForemanOverlayCoreTests", dependencies: ["ForemanOverlayCore"], resources: [
            .copy("Fixtures")
        ]),
        .testTarget(name: "ForemanOverlayUITests"),
    ]
)
