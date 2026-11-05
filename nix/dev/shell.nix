{lib, ...}: let
  inherit (lib) optionals;
in {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) pre-commit;
    inherit
      (pkgs)
      mkShell
      swift
      swiftpm
      ;
  in {
    devShells.default = mkShell.override {inherit (swift) stdenv;} {
      nativeBuildInputs =
        [swift swiftpm]
        ++ optionals swift.stdenv.hostPlatform.isLinux [pkgs.gitMinimal pkgs.swiftpm2nix]
        ++ pre-commit.settings.enabledPackages;

      buildInputs = optionals swift.stdenv.hostPlatform.isDarwin [
        pkgs.apple-sdk_26
        (pkgs.darwinMinVersionHook "26.0")
      ];

      shellHook = pre-commit.installationScript;
    };
  };
}
