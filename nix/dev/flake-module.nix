{inputs, ...}: let
  inherit (inputs) git-hooks;
  systems = import inputs.systems;
in {
  imports = [
    git-hooks.flakeModule
    ./shell.nix
    ./pre-commit.nix
  ];

  inherit systems;

  perSystem = {pkgs, ...}: {
    formatter = pkgs.alejandra;
  };
}
