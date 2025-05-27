{
  description = "a minimal WASM interpreter";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    typst-packages = {
      url = "github:typst/packages";
      flake = false;
    };
    typix = {
      url = "github:loqusion/typix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "git+https://github.com/numtide/flake-utils.git";
    devshell.url = "github:numtide/devshell";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
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
              (import inputs.rust-overlay)
            ];
          };

          # universal formatter
          treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

          # rust target name of the `system`
          rust-target = pkgs.pkgsStatic.targetPlatform.rust.rustcTarget;

          # parsed contents of Cargo.toml
          cargoToml = lib.trivial.importTOML ./Cargo.toml;

          # minimum rust version that we support according to Cargo.toml
          msrv = cargoToml.package.rust-version;

          # Rust distribution for our hostSystem
          rust-toolchain-nixpkgs-current = pkgs.rust-bin.stable.${pkgs.rustc.version}.default.override {
            extensions = [ "rust-src" ];
            targets = [
              rust-target
              "wasm32-unknown-unknown"
              "thumbv6m-none-eabi" # for no_std test
              "i686-unknown-linux-musl" # to test if we can run on 32 Bit architectures
            ];
          };

          # Rust toolchain for the MSRV
          rust-toolchain-msrv = pkgs.rust-bin.stable.${msrv}.default.override {
            extensions = [ "rust-src" ];
            targets = [
              rust-target
              "wasm32-unknown-unknown"
              "thumbv6m-none-eabi" # for no_std test
              "i686-unknown-linux-musl" # to test if we can run on 32 Bit architectures
            ];
          };

          # Typst packages for the whitepaper
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
          packages =
            {
              wasm-interpreter = pkgs.callPackage pkgs/wasm-interpreter.nix { };

              whitepaper = inputs.typix.lib.${system}.buildTypstProject {
                name = "whitepaper.pdf";
                src = ./whitepaper;
                XDG_CACHE_HOME = typstPackagesCache;
              };

              report = pkgs.callPackage pkgs/report.nix {
                inherit (self.packages.${system}) wasm-interpreter whitepaper;
              };

            }
            // lib.attrsets.optionalAttrs pkgs.hostPlatform.isx86 {
              # Interpreter package tailored for i686 build. As the upstream Nix binary cache does not
              # contain most 32 bit binaries anymore, we inject the oxalica toolchain into the x86_64
              # build, to avoid massive recompilation. Apparently cargo-nextest has a transitive depency
              # on Firefox, which would lead to days of rebuilding when using the Nix built-in cross
              # compilation mechanism.
              wasm-interpreter-cross-i686-linux =
                (self.packages.${system}.wasm-interpreter.override {
                  # for the 32 bit check we are fine if the tests pass, no need for documentation,
                  # benchmarking or coverage measurement
                  doDoc = false;
                  doBench = false;
                  doMeasureCoverage = false;
                }).overrideAttrs
                  (old: {
                    env = old.env // {
                      CARGO_BUILD_TARGET = "i686-unknown-linux-musl";
                    };
                  });
            };

          # a devshell with all the necessary bells and whistles
          devShells.default = (
            pkgs.devshell.mkShell {
              imports = [ "${devshell}/extra/git/hooks.nix" ];
              name = "wasm-interpreter";
              packagesFrom = [
                self.packages.${system}.report
                self.packages.${system}.whitepaper
              ];
              packages = with pkgs; [
                stdenv.cc
                coreutils
                rust-toolchain-nixpkgs-current
                rust-analyzer
                cargo-audit
                cargo-expand
                cargo-llvm-cov
                cargo-outdated
                cargo-udeps
                cargo-watch
                nodePackages.prettier
                wabt

                # utilities
                treefmtEval.config.build.wrapper
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
                pre-commit.text = "nix flake check '.?submodules=1'";
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
                  # TODO maybe accept the name of the target branch as an argument and use it instead of `main`
                  command = ''
                    (
                      cd $PRJ_ROOT
                      TESTSUITE_SAVE=1 cargo test -- spec_tests --show-output
                      cp testsuite_results.json new.json
                      mkdir .main_clone
                      git clone --depth 1 --single-branch --no-tags --recursive -b main $(git config --get remote.origin.url) .main_clone
                      cd .main_clone
                      TESTSUITE_SAVE=1  cargo test -- spec_tests --show-output
                      mv testsuite_results.json ../old.json
                      cd ..
                      rm -rf .main_clone
                      cargo run --manifest-path ./ci-tools/compare-testsuite-rs/Cargo.toml -- old.json new.json > testsuite_report.md
                    )
                  '';
                  help = "generates a comparison document for the official wasm testsuite w.r.t. project main branch";
                }
              ];
            }
          );

          # a simple devshell to compile with the MSRV
          devShells.msrv = pkgs.mkShell {
            inputsFrom = [ self.checks.${system}.wasm-interpreter-msrv ];
          };

          # for `nix fmt`
          formatter = treefmtEval.config.build.wrapper;

          # always check these
          checks = {
            # check that all files are properly formatted
            formatting = treefmtEval.config.build.check self;

            # check that the Minimum Supported Rust Version (MSRV) we promise does actually compile
            wasm-interpreter-msrv = self.packages.${system}.wasm-interpreter.override {
              # rustPlatform based on the MSRV we promise
              rustPlatform = pkgs.makeRustPlatform {
                cargo = rust-toolchain-msrv;
                rustc = rust-toolchain-msrv;
              };

              # we do neither need documentation nor mandate benchmarking or coverage measurement to
              # work for the MSRV
              doDoc = false;
              doBench = false;
              doMeasureCoverage = false;
              isMsrvCheck = true;
            };

            # check that the requirements can be parsed
            requirements = pkgs.runCommand "check-requirement" { nativeBuildInputs = [ pkgs.strictdoc ]; } ''
              shopt -s globstar
              strictdoc passthrough ${./.}/requirements/**.sdoc
              touch $out
            '';
          };
        }
      );
}
