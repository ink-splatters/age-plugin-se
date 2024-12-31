{
  mkShell,
  swift,
  swiftpm,
  swiftpm2nix,
  ...
}:
mkShell.override { inherit (swift) stdenv; } {
  nativeBuildInputs = [
    swift
    swiftpm
    swiftpm2nix
  ];
}
