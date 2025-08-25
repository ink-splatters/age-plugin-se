{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    config.packages.age-plugin-se = let
      generated = swiftpm2nix.helpers ./swiftpm2nix;
      inherit
        (pkgs)
        swift
        swiftpm
        swiftpm2nix
        ;
    in
      swift.stdenv.mkDerivation rec {
        pname = "age-plugin-se";
        version = "0.1.4+20250715";
        inherit (config) src;

        configurePhase = generated.configure;

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
      };
  };
}
