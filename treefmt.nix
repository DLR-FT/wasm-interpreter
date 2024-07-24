{ pkgs, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  #programs.actionlint.enable = true;
  programs.nixpkgs-fmt.enable = true;
  programs.prettier = {
    enable = true;
    includes = [
      "*.cjs"
      "*.css"
      "*.html"
      "*.js"
      "*.json"
      "*.json5"
      "*.jsx"
      "*.md"
      "*.mdx"
      "*.mjs"
      "*.scss"
      "*.toml"
      "*.ts"
      "*.tsx"
      "*.vue"
      "*.yaml"
      "*.yml"
    ];
    settings = {
      plugins = [
        "${pkgs.nodePackages.prettier-plugin-toml}/lib/node_modules/prettier-plugin-toml/lib/index.js"
      ];
    };
  };
  programs.rustfmt.enable = true;
}
