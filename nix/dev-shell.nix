{
  mkShell,
  swift,
  swiftformat,
  swiftpm,
  swiftpm2nix,
  ...
}:
mkShell.override {inherit (swift) stdenv;} {
  nativeBuildInputs = [
    swift
    swiftformat
    swiftpm
    swiftpm2nix
  ];
}
