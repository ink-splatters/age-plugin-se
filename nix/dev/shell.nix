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
      ;
  in {
    devShells.default = mkShell.override {inherit (swift) stdenv;} {
      nativeBuildInputs =
        [swift swiftpm]
        ++ pre-commit.settings.enabledPackages;

      shellHook = pre-commit.installationScript;
    };
  };
}
