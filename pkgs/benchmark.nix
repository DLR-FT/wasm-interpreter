{ wasm-interpreter }:

wasm-interpreter.overrideAttrs (old: {
  pname = old.pname + "-benchmark";
  cargoBuildFlags = [ "--package=benchmark" ];

  postBuild = ''
    pushd crates/benchmark
    cargo bench
    popd
  '';

  doCheck = false;

  # TODO add preInstall and postInstlal hook
  installPhase = ''
    runHook preInstall

    shopt -s globstar
    mv target/**/criterion/ "$out/"
    shopt -u globstar

    runHook postInstall
  '';
})
