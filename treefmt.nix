{ pkgs, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  programs.prettier = {
    enable = true;
    settings = {
      plugins = [
        "${pkgs.nodePackages.prettier-plugin-toml}/lib/node_modules/prettier-plugin-toml/lib/index.js"
      ];
    };
  };
}
