{
  lib,
  rustPlatform,
  clippy,
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = "rust-workspace";
  version = cargoToml.package.version;

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
      ];
      # Files to explicitly include
      include = [ ];
      # Files to explicitly exclude
      exclude = [
        "flake.lock"
        "treefmt.toml"
      ];

      filter = (
        path: type:
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
        ])
        && !(anyof [ (any (file: file == relative) exclude) ])
      );
    in
    lib.sources.cleanSourceWith { inherit src filter; };

  cargoLock.lockFile = src + "/Cargo.lock";

  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    RUSTDOCFLAGS="-Dwarnings" cargo doc --workspace --all-features --document-private-items
    RUSTFLAGS="-Dwarnings" ${clippy}/bin/cargo-clippy --workspace --all-features
  '';
  installPhase = ''
    mkdir "$out"
  '';

  meta = {
    description = "Checks for the Rust workspace";
    license = with lib.licenses; [
      asl20
      # OR
      mit
    ];
    maintainers = [ ];
  };
}
