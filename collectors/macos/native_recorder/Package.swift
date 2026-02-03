// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "aegis-native-recorder",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .executable(name: "aegis-native-recorder", targets: ["AegisNativeRecorder"])
    ],
    targets: [
        .executableTarget(
            name: "AegisNativeRecorder",
            dependencies: [],
            path: "Sources"
        )
    ]
)
