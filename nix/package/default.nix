{
  lib,
  swift,
  swiftpm,
  swiftpm2nix,
  ...
}: let
  pkg = {outputHash}:
    swift.stdenv.mkDerivation rec {
      pname = "age-plugin-se";
      version = "0.1.4";
      src = ../../.;

      nativeBuildInputs = [
        swift
        swiftpm
      ];

      installPhase = ''
        binPath="$(swiftpmBinPath)"
        mkdir -p $out/bin
        cp $binPath/${pname} $out/bin/
      '';

      enableParallelBuilding = true;

      inherit outputHash;
      outputHashMode = "recursive";
    };

  darwin-pkg = pkg;

  linux-pkg = {outputHash}: let
    generated = swiftpm2nix.helpers ./swiftpm2nix;
  in
    pkg.overrideAttrs (_oa: {
      configurePhase = generated.configure;
      inherit outputHash;
    });
in {
  "aarch64-darwin" = darwin-pkg {outputHash = "sha256-ghFZL78LiXCg/8OdNXLZGHpGg5Xh/WZqozGfBTmfr8c=";};
  "x86_64-darwin" = darwin-pkg {outputHash = lib.fakeHash;}; # TODO

  "aarch64-linux" = linux-pkg {outputHash = lib.fakeHash;}; # TODO
  "x86_64-linux" = linux-pkg {outputHash = lib.fakeHash;}; # TODO
}
