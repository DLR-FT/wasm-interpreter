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
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, utils, naersk, devshell, treefmt-nix, ... }@inputs:
    utils.lib.eachSystem [ "x86_64-linux" "i686-linux" "aarch64-linux" ] (system:
      let
        lib = nixpkgs.lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        # universal formatter
        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

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
          name = "wasm-interpreter";
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
            nodePackages.prettier
            strictdoc
            wabt

            # utilities
            nixpkgs-fmt
            nodePackages.prettier
            treefmtEval.config.build.wrapper
          ];
          git.hooks = {
            enable = true;
            pre-commit.text = "nix flake check";
          };
          commands = [
            {
              name = "requirements-export-excel";
              command = ''
                strictdoc export --output-dir "$PRJ_ROOT/requirements/export" \
                  --formats=excel \
                  "$PRJ_ROOT/requirements"
              '';
              help = "export the requirements to requirements/export";
            }
            {
              name = "requirements-export-html";
              command = ''
                strictdoc export --output-dir "$PRJ_ROOT/requirements/export" \
                  --formats=html \
                  "$PRJ_ROOT/requirements"
              '';
              help = "export the requirements to requirements/export";
            }
            {
              name = "requirements-web-server";
              command = ''
                strictdoc server "$PRJ_ROOT/requirements"
              '';
              help = "start the requirements editor web-ui";
            }
          ];
        });


        # for `nix fmt`
        formatter = treefmtEval.config.build.wrapper;

        # always check these
        checks = {
          formatting = treefmtEval.config.build.check self;
          # TODO remove once https://github.com/numtide/treefmt/issues/153 is closed
          format-bug-fix = pkgs.runCommand "yaml-fmt"
            {
              nativeBuildInputs = [ pkgs.nodePackages.prettier ];
            } "cd ${./.} && prettier --check .github; touch $out";

          requirements = pkgs.runCommand "check-requirement"
            {
              nativeBuildInputs = [ pkgs.strictdoc ];
            } ''
            shopt -s globstar
            strictdoc passthrough ${./.}/requirements/**.sdoc
            touch $out
          '';
        };
      });
}

