{
  description = "Age plugin for Apple's Secure Enclave";

  inputs = {
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  nixConfig = {
    extra-substituters = [
      "https://aarch64-darwin.cachix.org"
    ];
    extra-trusted-public-keys = [
      "aarch64-darwin.cachix.org-1:mEz8A1jcJveehs/ZbZUEjXZ65Aukk9bg2kmb0zL9XDA="
    ];
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} (let
      flakeModule = import ./nix/flake-module.nix {};
    in {
      imports = [
        flake-parts.flakeModules.partitions
        flakeModule
      ];

      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {config, ...}: {
        packages.default = config.packages.age-plugin-se;
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
        inherit flakeModule;
      };
    });
}
