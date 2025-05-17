{
  perSystem = {
    config,
    inputs',
    lib,
    pkgs,
    ...
  }: {
    options = let
      inherit (inputs'.fenix.packages) complete;
      inherit (complete) rustfmt cargo;
      inherit (lib) mkOption types;
      mkToolchain = components: complete.withComponents components;

      toolchain-components = [
        "cargo"
        "clippy"
        "rustc"
        "rustfmt"
      ];
    in {
      cargo = mkOption {
        type = types.package;
        default = cargo;
      };
      rustfmt = mkOption {
        type = types.package;
        default = rustfmt;
      };
      toolchain = mkOption {
        type = types.package;
        default = mkToolchain toolchain-components;
      };

      dev-toolchain = mkOption {
        type = types.package;
        default = mkToolchain (toolchain-components ++ ["rust-src"]);
      };

      rustPlatform = mkOption {
        type = types.attrs;
        default =
          (pkgs.makeRustPlatform {
            cargo = config.toolchain;
            rustc = config.toolchain;
          }).overrideScope (_: prev: {
            buildRustPackage = prev.buildRustPackage.override {
              inherit (pkgs.llvmPackages_latest) stdenv;
            };
          });
      };
    };
  };
}
