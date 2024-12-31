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
        version = "0.1.4";
        inherit (config) src;

        configurePhase = generated.configure;

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
