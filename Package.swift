// swift-tools-version: 5.8

import PackageDescription

let package = Package(
  name: "AgeSecureEnclavePlugin",
  platforms: [.macOS(.v13)],
  dependencies: [
    // Only used on Linux & Windows
    .package(url: "https://github.com/apple/swift-crypto.git", "2.0.0"..<"3.0.0"),
    .package(url: "https://github.com/ink-splatters/swift-testing-revolutionary", exact: "0.3.2+swift-tools-4.8")
  ],
  targets: [
    .executableTarget(
      name: "age-plugin-se",
      dependencies: [
        .product(
          name: "Crypto", package: "swift-crypto", condition: .when(platforms: [.linux, .windows]))
      ],
      path: "Sources"),
    .testTarget(name: "Tests", dependencies: ["age-plugin-se"], path: "Tests"),
  ]
)
