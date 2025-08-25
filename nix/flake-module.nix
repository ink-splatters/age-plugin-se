{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    config.packages.age-plugin-se = let
      inherit
        (pkgs)
        swift
        swiftpm
        ;
    in
      swift.stdenv.mkDerivation rec {
        pname = "age-plugin-se-macos";
        version = "0.1.4+20250825";
        inherit (config) src;

        nativeBuildInputs = [
          swift
          swiftpm
        ];

        installPhase = ''
          binPath="$(swiftpmBinPath)"
          mkdir -p $out/bin
          cp $binPath/age-plugin-se $out/bin/
        '';

        enableParallelBuilding = true;
      };
  };
}
