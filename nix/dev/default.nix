{inputs, ...}: let
  inherit (inputs) git-hooks;
in {
  imports = [
    git-hooks.flakeModule
    ./shell.nix
    ./pre-commit.nix
  ];

  perSystem = {pkgs, ...}: {
    formatter = pkgs.alejandra;
  };
}
