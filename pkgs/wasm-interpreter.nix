{ lib, rustPlatform, cargo-llvm-cov }:

let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src =
    let
      # original source to read from
      src = ./..;

      # File suffices to include
      extensions = [ "lock" "rs" "toml" ];
      # Files to explicitly include
      include = [ ];
      # Files to explicitly exclude
      exclude = [ "flake.lock" "treefmt.toml" ];

      filter = (path: type:
        let
          inherit (builtins) baseNameOf toString;
          inherit (lib.lists) any;
          inherit (lib.strings) hasSuffix removePrefix;
          inherit (lib.trivial) id;

          # consumes a list of bools, returns true if any of them is true
          anyof = any id;

          basename = baseNameOf (toString path);
          relative = removePrefix (toString src + "/") (toString path);
        in
        (anyof [
          (type == "directory")
          (any (ext: hasSuffix ".${ext}" basename) extensions)
          (any (file: file == relative) include)
        ]) && !(anyof [
          (any (file: file == relative) exclude)
        ])
      );
    in
    lib.sources.cleanSourceWith { inherit src filter; };

  cargoLock.lockFile = src + "/Cargo.lock";

  # we want a full documentation
  postBuild = ''
    cargo doc --document-private-items
    mkdir -- "$out"
    mv target/doc "$out/"
  '';

  # required to measure coverage
  nativeCheckInputs = [ cargo-llvm-cov ];
  env = { inherit (cargo-llvm-cov) LLVM_COV LLVM_PROFDATA; };

  # nextest can emit JUnit test reports
  useNextest = true;
  cargoTestFlags = [ "--profile=ci" ];

  # TODO `cargo llvm-cov report --doctest` is only available on nightly :(
  postCheck = ''
    # bench
    cargo bench
    mv target/criterion "$out/bench-html"

    # coverage stuff
    cargo llvm-cov --no-report nextest
    cargo llvm-cov report --lcov --output-path lcov.info
    cargo llvm-cov report --json --output-path lcov.json
    cargo llvm-cov report --cobertura --output-path lcov-cobertura.xml
    cargo llvm-cov report --codecov --output-path lcov-codecov.json
    cargo llvm-cov report --text --output-path lcov.txt
    cargo llvm-cov report --html --output-dir lcov-html
    mv lcov* target/nextest/ci/junit.xml "$out/"
  '';

  meta = {
    inherit (cargoToml.package) description homepage;
    license = with lib.licenses; [ asl20 /* OR */ mit ];
    maintainers = [ lib.maintainers.wucke13 ];
  };
}
