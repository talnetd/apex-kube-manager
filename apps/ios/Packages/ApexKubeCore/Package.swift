// swift-tools-version:5.10
// ApexKubeCore: platform-agnostic logic for the iOS app (no UIKit/SwiftUI).
// Later PRs add Kubeconfig, Credentials and KubeClient targets next to Triage.
import PackageDescription

let package = Package(
    name: "ApexKubeCore",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "Triage", targets: ["Triage"]),
    ],
    targets: [
        .target(name: "Triage"),
        .testTarget(name: "TriageTests", dependencies: ["Triage"]),
    ]
)
