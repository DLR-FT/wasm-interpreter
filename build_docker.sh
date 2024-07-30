docker build -t easy/nix .
docker run -it --rm -v $(pwd):/wasm-interpreter -p 1337:1337 easy/nix
