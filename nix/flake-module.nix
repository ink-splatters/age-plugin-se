{
  imports = [
    ./rust-platform.nix
  ];

  perSystem = {
    config,
    lib,
    pkgs,
    ...
  }: {
    packages.rage-plutin-se = let
      inherit (config) rustPlatform;
      inherit (pkgs.llvmPackages_latest) clang bintools;
    in
      rustPlatform.buildRustPackage {
        name = "rage-plugin-se";
        version = "0.1.0";

        src = ../.;
        useFetchCargoVendor = true;

        RUSTFLAGS = lib.concatMapStringsSep " " (x: "-C ${x}") [
          "codegen-units=1"
          "debuginfo=0"
          "embed-bitcode=yes"
          "linker=${clang}/bin/cc"
          "link-args=-fuse-ld=lld"
          "lto=full"
          "opt-level=3"
          "prefer-dynamic=no"
          "strip=symbols"
          "target-cpu=native"
        ];

        nativeBuildInputs = [
          clang
          bintools
        ];

        NIX_ENFORCE_NO_NATIVE = 0;
        NIX_ENFORCE_PURITY = 0;

        enableParallelBuilding = true;
      };
  };
}
