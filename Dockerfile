from ubuntu

RUN apt update && \
    apt install -y curl sudo xz-utils python3 git

RUN curl -L https://nixos.org/nix/install > install.sh && \
    chmod +x install.sh && \
    printf "n\ny\ny\n\n\n" | ./install.sh --daemon

RUN mkdir -p ~/.config/nix/ && \
    echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf

RUN git config --global --add safe.directory /wasm-interpreter
