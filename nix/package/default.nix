{
  swift,
  swiftpm,
  swiftpm2nix,
  ...
}:
let
  generated = swiftpm2nix.helpers ./swiftpm2nix;
in
swift.stdenv.mkDerivation rec {
  pname = "age-plugin-se";
  version = "0.1.4";
  src = ../../.;

  nativeBuildInputs = [
    swift
    swiftpm
  ];

  configurePhase = generated.configure;

  installPhase = ''
    binPath="$(swiftpmBinPath)"
    mkdir -p $out/bin
    cp $binPath/${pname} $out/bin/
  '';

  enableParallelBuilding = true;
}
