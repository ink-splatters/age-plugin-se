{
  pkgs,
  git-hooks,
  src,
  system,
  ...
}:
rec {
  check = git-hooks.lib.${system}.run {
    inherit src;

    hooks = {
      deadnix.enable = true;
      markdownlint.enable = true;
      nil.enable = true;
      nixfmt-rfc-style.enable = true;
      statix.enable = true;
      shellcheck.enable = true;
      shfmt.enable = true;
      # swift.enable = true;
    };

    tools = pkgs;
  };

  install-hooks = {
    type = "app";
    program = toString (
      pkgs.writeShellScript "install-hooks" ''
        ${check.shellHook}
        echo Done!
      ''
    );
    meta.description = "install pre-commit hooks";
  };
}
