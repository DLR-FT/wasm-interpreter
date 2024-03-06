{
  description = "a minimal WASM interpreter";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "git+https://github.com/numtide/flake-utils.git";
    devshell.url = "github:numtide/devshell";
    fenix = {
      url = "git+https://github.com/nix-community/fenix.git?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "git+https://github.com/nix-community/naersk.git";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, utils, naersk, devshell, ... }@inputs:
    utils.lib.eachSystem [ "x86_64-linux" "i686-linux" "aarch64-linux" ] (system:
      let
        lib = nixpkgs.lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        # rust target name of the `system`
        rust-target = pkgs.rust.toRustTarget pkgs.pkgsStatic.targetPlatform;

        # Rust distribution for our hostSystem
        fenix = inputs.fenix.packages.${system};

        rust-toolchain = with fenix;
          combine [
            latest.rustc
            latest.cargo
            latest.clippy
            latest.rustfmt
            targets.${rust-target}.latest.rust-std
            targets.thumbv6m-none-eabi.latest.rust-std # for no_std test
            targets.wasm32-unknown-unknown.latest.rust-std
          ];

        # overrides a naersk-lib which uses the stable toolchain expressed above
        naersk-lib = (naersk.lib.${system}.override {
          cargo = rust-toolchain;
          rustc = rust-toolchain;
        });

      in
      rec {
        # a devshell with all the necessary bells and whistles
        devShells.default = (pkgs.devshell.mkShell {
          imports = [ "${devshell}/extra/git/hooks.nix" ];
          name = "aa";
          packages = with pkgs; [
            stdenv.cc
            coreutils
            rust-toolchain
            rust-analyzer
            cargo-outdated
            cargo-udeps
            cargo-watch
            cargo-audit
            cargo-expand
            nixpkgs-fmt
            nodePackages.prettier
            wabt
          ];
          git.hooks = {
            enable = true;
            pre-commit.text = "nix flake check";
          };
        });

        # always check these
        checks = {
          nixpkgs-fmt = pkgs.runCommand "nixpkgs-fmt"
            {
              nativeBuildInputs = [ pkgs.nixpkgs-fmt ];
            } "nixpkgs-fmt --check ${./.}; touch $out";
          cargo-fmt = pkgs.runCommand "cargo-fmt"
            {
              nativeBuildInputs = [ rust-toolchain ];
            } "cd ${./.}; cargo fmt --check; touch $out";
        };
      });
}

