{ lib, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  settings.global.excludes = [
    "tests/specification/testsuite/*"
  ];
  programs.nixfmt.enable = true;
  programs.prettier = {
    enable = true;
    includes = [
      "*.css"
      "*.html"
      "*.js"
      "*.json"
      "*.json5"
      "*.md"
      "*.mdx"
      "*.yaml"
      "*.yml"
    ];
  };
  programs.rustfmt = {
    enable = true;
    edition = (lib.importTOML ./Cargo.toml).package.edition;
  };
  programs.taplo.enable = true; # formats TOML files
  programs.typstfmt.enable = true;
}
