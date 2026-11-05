{lib, ...}: {
  imports = [
    ./age-plugin-se.nix
    ./swift-version.nix
  ];

  options.build.native = lib.mkOption {
    type = lib.types.str;
    default = "native";
  };
}
