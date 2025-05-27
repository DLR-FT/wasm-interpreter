{ buildTypstProject, typst-packages-cache }:

buildTypstProject {
  name = "whitepaper.pdf";
  src = ./.;
  XDG_CACHE_HOME = typst-packages-cache;
}
