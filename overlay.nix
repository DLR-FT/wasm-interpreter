final: prev:
let
  inherit (prev) lib;

  # root dir of this flake
  flakeRoot = ./.;

  # all packages from the local tree
  wasm-interpreter-pkgs = lib.filesystem.packagesFromDirectoryRecursive {
    # a special callPackage variant that contains our flakeRoot
    callPackage = lib.callPackageWith (final // { inherit flakeRoot; });

    # local tree of packages
    directory = ./pkgs;
  };
in

{
  # https://github.com/NixOS/nixpkgs/pull/42637
  requireFile =
    args:
    (prev.requireFile args).overrideAttrs (_: {
      allowSubstitutes = true;
    });

  # custom namespace for packages from the local tree
  inherit wasm-interpreter-pkgs;
}
// wasm-interpreter-pkgs
