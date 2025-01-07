{
  swift,
  swiftpm,
  ...
}:
swift.stdenv.mkDerivation rec {
  pname = "age-plugin-se";
  version = "0.1.4-macos-only";
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

  __contentAddressed = true;
  outputHashTye = "sha256";
  outputHashMode = "recursive";
  outputHash = "sha256-+/AAL17GmNtZi/QiLmh2DBQ05Ti6dfFZ6tptpf2IkBc=";
}
