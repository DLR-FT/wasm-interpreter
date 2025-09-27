{ wasm-interpreter, cargo-flamegraph }:

wasm-interpreter.overrideAttrs (old: {
  pname = old.pname + "-flamegraph";
  nativeBuildInputs = old.nativeBuildInputs ++ [ cargo-flamegraph ];
  cargoBuildFlags = [ "--package=benchmark" ];

  postBuild = ''
    pushd crates/benchmark
    CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph --package benchmark --bench general_purpose \
      --deterministic -- --bench
    popd
  '';

  doCheck = false;

  installPhase = ''
    runHook preInstall
    mkdir -- "$out"
    mv -- flamegraph.svg "$out/"
    runHook postInstall
  '';
})
