{inputs, ...}: {
  imports = [
    inputs.git-hooks.flakeModule
    ./shell.nix
    ./pre-commit.nix
  ];

  perSystem = {pkgs, ...}: {
    formatter = pkgs.alejandra;
  };
}
