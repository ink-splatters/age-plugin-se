{
  perSystem = {
    pkgs,
    system,
    ...
  }: {
    config.packages.age-plugin-se = (pkgs.callPackage ./package {}).${system};
  };
}
