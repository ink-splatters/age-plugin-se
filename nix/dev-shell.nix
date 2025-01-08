_: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) pre-commit;
  in {
    devShells.default = with pkgs;
      mkShell.override {inherit (swift) stdenv;} {
        nativeBuildInputs = with pkgs.swift;
          [swift swiftpm swiftpm2nix]
          ++ pre-commit.settings.enabledPackages;

        shellHook = pre-commit.installationScript;
      };
  };
}
