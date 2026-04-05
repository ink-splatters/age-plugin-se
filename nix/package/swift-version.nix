{
  perSystem = {lib, ...}: let
    versionLine =
      lib.findFirst (line: lib.hasPrefix "let version = \"" line) null
      (lib.splitString "\n" (builtins.readFile ../../Sources/CLI.swift));
    versionMatch =
      if versionLine == null
      then throw "Could not find CLI version in Sources/CLI.swift"
      else builtins.match ''let version = "([^"]+)"'' versionLine;
  in {
    options.version = lib.mkOption {
      type = lib.types.str;
      readOnly = true;
      description = "Version parsed from Sources/CLI.swift";
    };

    config.version =
      if versionMatch == null
      then throw "Could not parse CLI version from Sources/CLI.swift"
      else lib.removePrefix "v" (builtins.elemAt versionMatch 0);
  };
}
