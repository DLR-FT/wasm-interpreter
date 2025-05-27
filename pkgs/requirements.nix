{
  runCommand,
  strictdoc,
  flakeRoot,
}:

runCommand "compile-requirements" { nativeBuildInputs = [ strictdoc ]; } ''
  strictdoc export --formats html,json --enable-mathjax ${flakeRoot + "/requirements"}
  mv -- output "$out"
''
