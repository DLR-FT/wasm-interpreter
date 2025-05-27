{ stdenvNoCC, fetchFromGitHub }:

stdenvNoCC.mkDerivation {
  name = "typst-packages-cache";
  src = fetchFromGitHub {
    owner = "typst";
    repo = "packages";
    rev = "aa0d7b808aa3999f6e854f7800ada584b8eee0fa"; # updated on 2025-05-27
    hash = "sha256-pqqiDSXu/j6YV3RdYHPzW15jBKAhdUtseCH6tLMRizg=";
  };
  dontBuild = true;
  installPhase = ''
    mkdir --parent -- "$out/typst/packages"
    cp --dereference --no-preserve=mode --recursive --reflink=auto \
      --target-directory="$out/typst/packages" -- "$src"/packages/*
  '';
}
