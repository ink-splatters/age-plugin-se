_: {config, ...}: {
  perSystem = {
    lib,
    pkgs,
    ...
  }: let
    inherit (pkgs) swiftpm swiftpm2nix;
    inherit (pkgs.swift) stdenv swift;
  in {
    config.packages.age-plugin-se = stdenv.mkDerivation rec {
      pname = "age-plugin-se";
      version = "0.1.4";
      src = ../.;

      nativeBuildInputs = [swift swiftpm];

      configurePhase =
        lib.optionalString (!stdenv.hostPlatform.isDarwin)
        (swiftpm2nix.helpers ./swiftpm2nix).configure;

      installPhase = ''
        binPath="$(swiftpmBinPath)"
        mkdir -p $out/bin
        cp $binPath/${pname} $out/bin/
      '';

      enableParallelBuilding = true;

      outputHashMode = "recursive";
      outputHash = "sha256-OySBeUUk9ryYGEXvuBTC5G9ccsylLlq9M59+4DffHt8=";
    };
  };
}
