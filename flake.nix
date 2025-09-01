{
  description = "a minimal WASM interpreter";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
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
    {
      overlays.default = import ./overlay.nix;
    }
    //
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
                # import oxalica's rust overlay for version specific Rust toolchains
                (import inputs.rust-overlay)

                # inject dependencies for our overlay.nix
                (final: prev: {
                  inherit (inputs.typix.lib.${system}) buildTypstProject;
                })

                # import our overlay for the package in pkgs/
                self.overlays.default

                # add the devshell overlay for the devshell defined below
                devshell.overlays.default
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
          in
          {
            # packages from `pkgs/`, injected into the `pkgs` via our `overlay.nix`
            packages = pkgs.wasm-interpreter-pkgs;

            # a devshell with all the necessary bells and whistles
            devShells.default = (
              pkgs.devshell.mkShell {
                imports = [ "${devshell}/extra/git/hooks.nix" ];
                name = "wasm-interpreter";
                packagesFrom = [
                  self.packages.${system}.report
                  self.packages.${system}.requirements
                  self.packages.${system}.whitepaper
                ];
                packages = with pkgs; [
                  stdenv.cc
                  coreutils
                  rust-toolchain-nixpkgs-current # also contains clippy
                  rust-analyzer
                  cargo-audit
                  cargo-expand
                  cargo-flamegraph
                  cargo-llvm-cov
                  cargo-outdated
                  cargo-watch
                  critcmp # compare criterion.rs benchmark results
                  wabt

                  # utilities
                  treefmtEval.config.build.wrapper
                ];
                env = [
                  {
                    name = "LLVM_COV";
                    value = self.packages.${system}.coverage.LLVM_COV;
                  }
                  {
                    name = "LLVM_PROFDATA";
                    value = self.packages.${system}.coverage.LLVM_PROFDATA;
                  }
                ];
                git.hooks = {
                  enable = true;
                  pre-commit.text = "nix flake check '.?submodules=1'";
                };
                commands = [
                  {
                    name = "bench-against-main";
                    command = ''
                      cd -- "$PRJ_ROOT"
                      COMMON_BENCH_CMD=(cargo bench --package benchmark --bench general_purpose --)

                      # clone main branch of this repo to temporary copy; with trap based self clean-up
                      trap -- "rm --recursive --force -- \"$PRJ_ROOT/.main_clone\"" EXIT
                      git clone --depth=1 --single-branch --no-tags --recursive --branch=main -- "file://$PRJ_ROOT" "$PRJ_ROOT/.main_clone"

                      # bench main
                      pushd -- .main_clone
                      "''${COMMON_BENCH_CMD[@]}" --save-baseline benchmark-main.baseline
                      popd

                      # bench current
                      "''${COMMON_BENCH_CMD[@]}" --save-baseline benchmark-current.baseline

                      # copy results from current to main clone dir
                      find target/ -type d -name 'benchmark-current.baseline' -exec cp --recursive --no-target-directory -- {} .main_clone/{} \;

                      # compare results
                      critcmp --target-dir .main_clone/target -- "benchmark-main.baseline" "benchmark-current.baseline"
                    '';
                    help = "benchmark the current HEAD against the main branch";
                    category = "benchmark";
                  }
                  {
                    name = "bench-flamegraph";
                    command = ''
                      cd -- "$PRJ_ROOT"
                      CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph --package benchmark --bench general_purpose -- --bench
                    '';
                    help = "run the benchmarks and draw a flamegraph";
                    category = "benchmark";
                  }
                  {
                    name = "requirements-export-excel";
                    command = ''
                      strictdoc export --output-dir "$PRJ_ROOT/requirements/export" \
                        --formats=excel \
                        "$PRJ_ROOT/requirements"
                    '';
                    help = "export the requirements to requirements/export";
                    category = "requirements";
                  }
                  {
                    name = "requirements-export-html";
                    command = ''
                      strictdoc export --output-dir "$PRJ_ROOT/requirements/export" \
                        --formats=html \
                        "$PRJ_ROOT/requirements"
                    '';
                    help = "export the requirements to requirements/export";
                    category = "requirements";
                  }
                  {
                    name = "requirements-web-server";
                    command = ''
                      strictdoc server "$PRJ_ROOT/requirements"
                    '';
                    help = "start the requirements editor web-ui";
                    category = "requirements";
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
                      typst watch --root "$PRJ_ROOT/pkgs/whitepaper" "$PRJ_ROOT/pkgs/whitepaper/main.typ"
                    '';
                    help = "start typst watch loop for the whitepaper";
                  }
                  {
                    name = "generate-testsuite-report";
                    # TODO maybe accept the name of the target branch as an argument and use it instead of `main`
                    command = ''
                      cd -- "$PRJ_ROOT"

                      TESTSUITE_SAVE=1 cargo test -- spec_tests --show-output
                      cp -- testsuite_results.json new.json

                      trap -- "rm --recursive --force -- \"$PRJ_ROOT/.main_clone\"" EXIT
                      git clone --depth=1 --single-branch --no-tags --recursive --branch=main -- "file://$PRJ_ROOT" "$PRJ_ROOT/.main_clone"

                      pushd -- .main_clone
                      TESTSUITE_SAVE=1 cargo test -- spec_tests --show-output
                      mv -- testsuite_results.json ../old.json
                      popd

                      cargo run --package=compare-testsuite-rs -- old.json new.json > testsuite_report.md
                    '';
                    help = "generates a comparison document for the official wasm testsuite w.r.t. project main branch";
                  }
                ];
              }
            );

            # a simple devShell to compile with the MSRV
            devShells.msrv = pkgs.mkShell {
              inputsFrom = [ self.checks.${system}.wasm-interpreter-msrv ];
            };

            # a devShell for nightly based commands
            #
            #
            # Run this to get advanced coverage information
            # RUST_LOG=trace cargo llvm-cov --branch --doctests --release
            #
            #
            # Run this to find unsafe `unsafe`:
            # cargo miri test
            #
            #
            # Run this to find unused dependencies:
            # cargo udeps --workspace --benches --tests
            devShells.nightly = pkgs.mkShell {
              inputsFrom = [ self.checks.${system}.wasm-interpreter-msrv ];
              nativeBuildInputs = with pkgs; [
                ((rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)).override {
                  extensions = [
                    "miri-preview"
                    "rust-analysis"
                    "rust-src"
                  ];
                })
                cargo-llvm-cov
                cargo-udeps
              ];
              env = { inherit (pkgs.cargo-llvm-cov) LLVM_COV LLVM_PROFDATA; };
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

                # we do neither need documentation nor a JUnit test report
                doDoc = false;
                useNextest = false;
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
