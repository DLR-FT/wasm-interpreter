{
  lib,
  stdenvNoCC,
  python3Packages,
  strictdoc,
  wasm-interpreter,
  whitepaper,
}:

let
  evidenceRoot = lib.strings.escapeShellArg wasm-interpreter;
in
stdenvNoCC.mkDerivation {
  pname = wasm-interpreter.pname + "-report";
  version = wasm-interpreter.version;
  dontUnpack = true;

  nativeBuildInputs = [
    python3Packages.junit2html
    strictdoc
  ];

  installPhase = ''
    runHook preInstall

    mkdir -- "$out"
    cd "$out"

    cp --recursive -- ${evidenceRoot}/bench-html bench
    cp --recursive -- ${evidenceRoot}/lcov-html coverage
    cp --recursive -- ${evidenceRoot}/share/doc/ rustdoc
    cp --dereference -- ${whitepaper} whitepaper.pdf

    mkdir test
    junit2html ${evidenceRoot}/junit.xml test/index.html

    strictdoc export --formats html,json --enable-mathjax ${../requirements}
    mv output requirements

    cp ${./report_index.html} index.html

    runHook postInstall
  '';
}
