{
  lib,
  rustPlatform,
  doDoc ? true,
  useNextest ? true,
}:

let
  cargoToml = fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.workspace.package.version; # get the version from the workspace because we do package.version = { workspace = true }

  src =
    let
      # original source to read from
      src = ./..;

      # File suffices to include
      extensions = [
        "lock"
        "rs"
        "toml"
        "wast"
        "wasm"
      ];
      # Files to explicitly include
      include = [ "README.md" ];
      # Files to explicitly exclude
      exclude = [
        "flake.lock"
        "treefmt.toml"
      ];

      filter = (
        path: type:
        let
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
        ])
        && !(anyof [ (any (file: file == relative) exclude) ])
      );
    in
    lib.sources.cleanSourceWith { inherit src filter; };

  cargoLock.lockFile = src + "/Cargo.lock";

  # we want a full documentation, if at all
  postBuild = lib.strings.optionalString doDoc ''
    # Rationale for excluding `benchmark`: It contains only binaries and has a lot of dependencies.
    cargo doc --workspace --exclude benchmark --all-features --document-private-items --locked --offline --frozen
    mkdir -- "$out"

    shopt -s globstar
    mv -- target/**/doc "$out/"
    shopt -u globstar
  '';
  env = lib.attrsets.optionalAttrs doDoc {
    RUSTFLAGS = "-Dwarnings";
  };

  # nextest can emit JUnit test reports
  inherit useNextest;

  # if using nextest, use the ci profile
  cargoTestFlags = lib.lists.optional useNextest "--profile=ci";

  # run tests in debug mode
  checkType = "debug";

  # if using nextest, it will create a junit.xml
  postCheck = lib.strings.optionalString useNextest ''
    shopt -s globstar
    mv -- target/**/nextest/ci/junit.xml "$out/" 2> /dev/null \
      && echo 'installed junit.xml' || true
    shopt -u globstar
  '';

  meta = {
    inherit (cargoToml.package) description homepage;
    license = with lib.licenses; [
      asl20
      # OR
      mit
    ];
    maintainers = [ lib.maintainers.wucke13 ];
  };
}
