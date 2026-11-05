top: {
  perSystem = {pkgs, ...}: let
    inherit (pkgs) lib;
    inherit
      (pkgs)
      swift
      swiftpm
      ;
    mkPackage = extra:
      swift.stdenv.mkDerivation (rec {
          pname = "age-plugin-se";
          inherit (top.config) src version;

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
              -Xcc -mcpu=${top.config.build.native} \
              -Xcc -O3 \
              -Xswiftc -cross-module-optimization
            runHook postBuild
          '';
        }
        // extra);
  in {
    config.packages = lib.mkMerge [
      (lib.mkIf swift.stdenv.hostPlatform.isDarwin {
        age-plugin-se = mkPackage {
          # CryptoKit supplies crypto on Darwin. Strip SwiftPM's unused package
          # graph and lockfile only from the writable Nix build source.
          patches = [./darwin-no-swift-crypto.patch];
          postPatch = ''
            rm Package.resolved
          '';

          buildInputs = [
            pkgs.apple-sdk_26
            (pkgs.darwinMinVersionHook "26.0")
          ];
        };
      })
      (lib.mkIf swift.stdenv.hostPlatform.isLinux {
        age-plugin-se = mkPackage {
          configurePhase = (pkgs.swiftpm2nix.helpers ./swiftpm2nix).configure;
        };
      })
    ];
  };
}
