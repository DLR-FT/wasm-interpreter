{ rustPlatform, fetchFromGitHub }:

rustPlatform.buildRustPackage {

  name = "polybench-rs";
  version = "unstable-2020-11-22";

  src = fetchFromGitHub {
    owner = "JRF63";
    repo = "polybench-rs";
    rev = "babdd974bf92f03fbe4b2146ca356b4561f447d7";
    hash = "sha256-iHHdDeQ4de+bniGwKvmzrKiPmDuUdMctoGxZNfnXmbE=";
  };

  postPatch = ''
    findAndSubstitute(){
      grep --fixed-strings --files-with-matches --null --recursive -- "$1" . |
        while IFS= read -r -d "" FILE
        do
          substituteInPlace "$FILE" --replace-fail "$1" "$2"
        done
    }

    # this feature has been stabilized by now, no nightly required
    findAndSubstitute '#![feature(min_const_generics)]' ""

    # the benchmark errors out with:
    # values of the type `[Array1D<f64, 20000>; 20000]` are too big for the target architecture
    findAndSubstitute 'bench_and_print::<20000>();' ""
  '';
  cargoHash = "sha256-/kR4hthqGqnz3bi7NpAG3/nN4TBXG9KAp6kmUdPO7r0=";

  # ensure DWARF debug info is present
  cargoBuildFlags = [
    "--config profile.release.debug=true"
  ];

  # TODO re-asses this hack
  buildPhase = ''
    cargo build --release --target wasm32-unknown-unknown $cargoBuildFlags
  '';
  installPhase = ''
    mkdir -- $out
    cp --recursive -- target/wasm32-unknown-unknown/release/*.wasm $out
  '';
}
