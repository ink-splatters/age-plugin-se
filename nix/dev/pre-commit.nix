_: {
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
        # shellcheck.enable = true;
        # shfmt = {
        #   enable = true;
        #   excludes = ["search-commits-by-fingerprint.sh"];
        # };
        # swift.enable = true;
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
