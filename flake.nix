{
  description = "Description for the project";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-analyzer-src.follows = "";
      };
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} (let
      flakeModules.default = import ./nix/flake-module.nix;
      systems = import inputs.systems;
    in {
      imports = [
        flake-parts.flakeModules.partitions
        ./nix/flake-module.nix
      ];
      inherit systems;
      debug = true;
      perSystem = {config, ...}: {
        packages.default = config.packages.rage-plugin-se;
      };

      partitionedAttrs = {
        apps = "dev";
        checks = "dev";
        devShells = "dev";
        formatter = "dev";
      };
      partitions.dev = {
        extraInputsFlake = ./nix/dev;
        module = {
          imports = [./nix/dev/flake-module.nix];
        };
      };
      flake = {
        inherit flakeModules;
      };
    });
}
