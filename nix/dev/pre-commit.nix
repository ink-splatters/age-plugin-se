{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    pre-commit = {
      check.enable = true;

      settings.hooks = {
        deadnix.enable = true;
        # markdownlint.enable = true;
        nil.enable = true;
        alejandra.enable = true;
        statix.enable = true;

        check-toml.enable = true;
        taplo.enable = true;

        rustfmt = {
          enable = true;
          packageOverrides = {
            inherit (config) rustfmt cargo;
          };
        };
      };
    };

    apps.install-hooks = {
      type = "app";
      program = toString (pkgs.writeShellScript "install-hooks" ''
        ${config.pre-commit.installationScript}
        echo Done!
      '');
      meta.description = "install pre-commit hooks";
    };
  };
}
