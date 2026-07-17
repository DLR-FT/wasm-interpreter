{
  stdenvNoCC,
  python3Packages,
  dlr-wasm-interpreter-pkgs,
}:

stdenvNoCC.mkDerivation {
  pname = dlr-wasm-interpreter-pkgs.dlr-wasm-interpreter.pname + "-report";
  version = dlr-wasm-interpreter-pkgs.dlr-wasm-interpreter.version;
  dontUnpack = true;

  nativeBuildInputs = [
    python3Packages.junit2html
  ];

  installPhase = ''
    runHook preInstall

    mkdir -- "$out"
    pushd "$out"

    cp --recursive -- ${dlr-wasm-interpreter-pkgs.benchmark} bench
    cp --recursive -- ${dlr-wasm-interpreter-pkgs.coverage}/lcov-html coverage
    cp --recursive -- ${dlr-wasm-interpreter-pkgs.requirements} requirements
    cp --recursive -- ${
      dlr-wasm-interpreter-pkgs.dlr-wasm-interpreter.override { doDoc = true; }
    }/share/doc/ rustdoc
    cp --dereference -- ${dlr-wasm-interpreter-pkgs.whitepaper} whitepaper.pdf

    mkdir test
    junit2html ${
      dlr-wasm-interpreter-pkgs.dlr-wasm-interpreter.override { useNextest = true; }
    }/junit.xml test/index.html


    cp ${./report_index.html} index.html

    popd

    runHook postInstall
  '';
}
