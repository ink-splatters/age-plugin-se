{
  imports = [
    ./swift-version.nix
  ];

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
        pname = "age-plugin-se";
        inherit (config) src version;

        nativeBuildInputs = [
          swift
          swiftpm
        ];

        installPhase = ''
          mkdir -p $out/bin
          cp .build/release/${pname} $out/bin/
        '';

        enableParallelBuilding = true;

        NIX_ENFORCE_NO_NATIVE = 0;

        buildPhase = ''
          runHook preBuild
          swift build -c release \
            -Xcc -mcpu=native \
            -Xcc -O3 \
            -Xswiftc -cross-module-optimization
          runHook postBuild
        '';
      };
  };
}
