{
  description = "a minimal WASM interpreter";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    typst-packages = {
      url = "github:typst/packages";
      flake = false;
    };
    typix = {
      url = "github:loqusion/typix";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
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
  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      devshell,
      treefmt-nix,
      ...
    }@inputs:
    utils.lib.eachSystem
      [
        "x86_64-linux"
        "i686-linux"
        "aarch64-linux"
      ]
      (
        system:
        let
          lib = nixpkgs.lib;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              devshell.overlays.default

              # We unfortunately need the most up-to-date typst
              (final: prev: { typst = inputs.nixpkgs-unstable.legacyPackages.${pkgs.hostPlatform.system}.typst; })
            ];
          };

          # universal formatter
          treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

          # rust target name of the `system`
          rust-target = pkgs.pkgsStatic.targetPlatform.rust.rustcTarget;

          # Rust distribution for our hostSystem
          fenix = inputs.fenix.packages.${system};

          rust-toolchain =
            with fenix;
            combine [
              latest.rustc
              latest.cargo
              latest.clippy
              latest.rustfmt
              targets.${rust-target}.latest.rust-std
              targets.thumbv6m-none-eabi.latest.rust-std # for no_std test
              targets.wasm32-unknown-unknown.latest.rust-std
              targets.i686-unknown-linux-musl.latest.rust-std # to test if we can run on 32 Bit architectures
            ];

          # overrides a naersk-lib which uses the stable toolchain expressed above
          naersk-lib = (
            naersk.lib.${system}.override {
              cargo = rust-toolchain;
              rustc = rust-toolchain;
            }
          );

          typstPackagesCache = pkgs.stdenv.mkDerivation {
            name = "typst-packages-cache";
            src = inputs.typst-packages;
            dontBuild = true;
            installPhase = ''
              mkdir -p "$out/typst/packages"
              cp --dereference --no-preserve=mode --recursive --reflink=auto \
                --target-directory="$out/typst/packages" -- "$src"/packages/*
            '';
          };
        in
        {
          # packages
          packages.wasm-interpreter = pkgs.callPackage pkgs/wasm-interpreter.nix { };
          packages.whitepaper = inputs.typix.lib.${system}.buildTypstProject {
            name = "whitepaper.pdf";
            src = ./whitepaper;
            XDG_CACHE_HOME = typstPackagesCache;
          };

          packages.report = pkgs.callPackage pkgs/report.nix {
            inherit (self.packages.${system}) wasm-interpreter whitepaper;
          };

          # a devshell with all the necessary bells and whistles
          devShells.default = (
            pkgs.devshell.mkShell {
              imports = [ "${devshell}/extra/git/hooks.nix" ];
              name = "wasm-interpreter";
              packages = with pkgs; [
                stdenv.cc
                coreutils
                rust-toolchain
                rust-analyzer
                cargo-audit
                cargo-expand
                cargo-llvm-cov
                cargo-outdated
                cargo-udeps
                cargo-watch
                nodePackages.prettier
                strictdoc
                wabt

                # utilities
                nixpkgs-fmt
                nodePackages.prettier
                treefmtEval.config.build.wrapper
                typst # for the whitepaper
                python3 # for comparing official testsuite results
              ];
              env = [
                {
                  name = "LLVM_COV";
                  value = self.packages.${system}.wasm-interpreter.LLVM_COV;
                }
                {
                  name = "LLVM_PROFDATA";
                  value = self.packages.${system}.wasm-interpreter.LLVM_PROFDATA;
                }
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
                {
                  name = "cargo-watch-doc";
                  command = ''
                    cargo watch --shell 'cargo doc --document-private-items'
                  '';
                  help = "start cargo watch loop for documentation";
                }
                {
                  name = "whitepaper-watch";
                  command = ''
                    typst watch --root "$PRJ_ROOT/whitepaper" "$PRJ_ROOT/whitepaper/main.typ"
                  '';
                  help = "start typst watch loop for the whitepaper";
                }
                {
                  name = "generate-testsuite-report";
                  command = ''
                    (
                      cd $PRJ_ROOT
                      TESTSUITE_SAVE=1 cargo test -- spec_tests --show-output
                      cp testsuite_results.json new.json
                      mkdir .main_clone
                      git clone --depth 1 --single-branch --no-tags --recursive -b dev/testsuite-preview $(git config --get remote.origin.url) .main_clone
                      cd .main_clone
                      TESTSUITE_SAVE=1  cargo test -- spec_tests --show-output
                      mv testsuite_results.json ../old.json
                      cd ..
                      rm -rf .main_clone
                      python3 ./ci_tools/compare_testsuite.py old.json new.json > testsuite_report.md
                    )
                  '';
                  help = "generates a comparison document for the official wasm testsuite w.r.t. project main branch";
                }
              ];
            }
          );

          # for `nix fmt`
          formatter = treefmtEval.config.build.wrapper;

          # always check these
          checks = {
            formatting = treefmtEval.config.build.check self;
            # TODO remove once https://github.com/numtide/treefmt/issues/153 is closed
            format-bug-fix = pkgs.runCommand "yaml-fmt" {
              nativeBuildInputs = [ pkgs.nodePackages.prettier ];
            } "cd ${./.} && prettier --check .github; touch $out";

            requirements = pkgs.runCommand "check-requirement" { nativeBuildInputs = [ pkgs.strictdoc ]; } ''
              shopt -s globstar
              strictdoc passthrough ${./.}/requirements/**.sdoc
              touch $out
            '';
          };
        }
      );
}
