{ wasm-interpreter, cargo-llvm-cov }:

wasm-interpreter.overrideAttrs (old: {
  pname = old.pname + "-coverage";

  nativeCheckInputs = [ cargo-llvm-cov ];

  env = {
    inherit (cargo-llvm-cov) LLVM_COV LLVM_PROFDATA;
  };

  dontBuild = true;

  checkPhase = ''
    runHook preCheck

    RUST_LOG=trace
    cargo llvm-cov --no-report nextest --features log/max_level_off

    runHook postCheck
  '';

  installPhase = ''
    runHook preInstall

    cargo llvm-cov report --lcov --output-path lcov.info
    cargo llvm-cov report --json --output-path lcov.json
    cargo llvm-cov report --cobertura --output-path lcov-cobertura.xml
    cargo llvm-cov report --codecov --output-path lcov-codecov.json
    cargo llvm-cov report --text --output-path lcov.txt
    cargo llvm-cov report --html --output-dir lcov-html

    mkdir --parent -- "$out"
    mv lcov* "$out/"

    runHook postInstall
  '';
})
