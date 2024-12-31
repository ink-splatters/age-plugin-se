{
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
      swiftpm2nix
      ;
  in {
    devShells.default = mkShell.override {inherit (swift) stdenv;} {
      nativeBuildInputs =
        [swift swiftpm swiftpm2nix]
        ++ pre-commit.settings.enabledPackages;

      shellHook = pre-commit.installationScript;
    };
  };
}
