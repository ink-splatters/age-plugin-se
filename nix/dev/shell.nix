{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    devShells.default = let
      inherit (config) pre-commit;
    in
      pkgs.mkShell.override {inherit (pkgs.llvmPackages_latest) stdenv;} {
        nativeBuildInputs =
          pre-commit.settings.enabledPackages
          ++ [config.dev-toolchain];

        shellHook = pre-commit.installationScript;
      };
  };
}
