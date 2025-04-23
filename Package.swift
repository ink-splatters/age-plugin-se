// swift-tools-version: 5.8

import PackageDescription

let package = Package(
  name: "AgeSecureEnclavePlugin",
  platforms: [.macOS(.v13)],
  dependencies: [
    // Only used on Linux & Windows
    .package(url: "https://github.com/apple/swift-crypto.git", "2.0.0"..<"4.0.0")
  ],
  targets: [
    .executableTarget(
      name: "age-plugin-se",
      dependencies: [
        .product(
          name: "Crypto", package: "swift-crypto", condition: .when(platforms: [.linux, .windows]))
      ],
    ),
    .testTarget(name: "Tests", dependencies: ["age-plugin-se"],),
  ]
)
