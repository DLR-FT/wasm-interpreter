{
  stdenvNoCC,
  python3Packages,
  strictdoc,
  flakeRoot,
  wasm-interpreter-pkgs,
}:

stdenvNoCC.mkDerivation {
  pname = wasm-interpreter-pkgs.wasm-interpreter.pname + "-report";
  version = wasm-interpreter-pkgs.wasm-interpreter.version;
  dontUnpack = true;

  nativeBuildInputs = [
    python3Packages.junit2html
    strictdoc
  ];

  installPhase = ''
    runHook preInstall

    mkdir -- "$out"
    pushd "$out"

    cp --recursive -- ${wasm-interpreter-pkgs.benchmark} bench
    cp --recursive -- ${wasm-interpreter-pkgs.coverage}/lcov-html coverage
    cp --recursive -- ${
      wasm-interpreter-pkgs.wasm-interpreter.override { doDoc = true; }
    }/share/doc/ rustdoc
    cp --dereference -- ${wasm-interpreter-pkgs.whitepaper} whitepaper.pdf

    mkdir test
    junit2html ${
      wasm-interpreter-pkgs.wasm-interpreter.override { useNextest = true; }
    }/junit.xml test/index.html

    strictdoc export --formats html,json --enable-mathjax ${flakeRoot + "/requirements"}
    mv output requirements

    cp ${./report_index.html} index.html

    popd

    runHook postInstall
  '';
}
