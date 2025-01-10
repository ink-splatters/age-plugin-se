{
  description = "Age plugin for Apple's Secure Enclave";

  inputs = {
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-compat.follows = "flake-compat";
      };
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
  };

  nixConfig = {
    extra-substituters = [
      "https://aarch64-darwin.cachix.org"
      "https://pre-commit-hooks.cachix.org"
    ];
    extra-trusted-public-keys = [
      "aarch64-darwin.cachix.org-1:mEz8A1jcJveehs/ZbZUEjXZ65Aukk9bg2kmb0zL9XDA="
      "pre-commit-hooks.cachix.org-1:Pkk3Panw5AW24TOv6kz3PvLhlH8puAsJTBbOPmBo7Rc="
    ];
  };

  outputs = inputs @ {
    git-hooks-nix,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} ({
      withSystem,
      flake-parts-lib,
      ...
    }: let
      pre-commit = import ./nix/pre-commit.nix {};
      dev-shell = import ./nix/dev-shell.nix {};
      systems = import inputs.systems;

      inherit (flake-parts-lib) importApply;
      flakeModule = importApply ./nix/flake-module.nix {inherit withSystem;};
    in {
      imports = [
        git-hooks-nix.flakeModule
        pre-commit
        dev-shell
        flakeModule
      ];

      inherit systems;

      perSystem = {
        config,
        pkgs,
        ...
      }: {
        formatter = pkgs.alejandra;
        packages.default = config.packages.age-plugin-se;
      };

      flake = {
        inherit flakeModule;
      };
    });
}
