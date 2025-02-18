#!/usr/bin/env bash

echo "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-="
echo "This script is not actively maintained by the"
echo "development team. Please run at your own risk."
echo "Feel free to report any issues on our github,"
echo "but we offer no guarantees of a fix being"
echo "implemented."
echo "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-="

if ! grep -q WSL /proc/version && grep -q MINGW /proc/version 2>/dev/null; then
    echo "This script requires WSL if ran on windows for proper path handling with Docker."
    echo "Please run this script from within WSL instead of Git Bash."
    exit 1
fi


# Make sure that we are in the `ci-tools` directory
current_dir=$(basename "$(pwd)")
if [ "$current_dir" != "ci-tools" ]; then
    if [ -d "ci-tools" ]; then
        cd "ci-tools" || exit 1
    else
        echo "Error: ci-tools dir not found -- where are you running this from?"
        exit 1
    fi
fi

mkdir -p nix-sharedfs
docker build -t nix-playground .

# Copy the parent directory contents into the container's workspace. This way, any results do not backspill onto the
# host which can mess up wsl/windows. We want to copy only the things git would copy (so we don't copy over `.git` or
# `target` or anything else git wouldn't copy)

temp_tar=$(mktemp)
cd ..
git archive --format=tar HEAD > "$temp_tar"
cd - || exit 1

CONTAINER_NAME="nix-playground-container"

if ! docker container inspect "$CONTAINER_NAME" >/dev/null 2>&1; then
    echo "Creating new container..."

    docker run -dit \
        --name "$CONTAINER_NAME" \
        -v "$(pwd)/nix-sharedfs:/nix-sharedfs" \
        --workdir /workspace \
        nix-playground \
        bash -c "mkdir -p /workspace && cd /workspace && bash"
else
    echo "Container exists, updating workspace and starting..."
    docker start "$CONTAINER_NAME"
fi

docker cp "$temp_tar" "$CONTAINER_NAME:/tmp/repo.tar"
docker exec "$CONTAINER_NAME" bash -c 'mkdir -p /workspace && cd /workspace && tar xf /tmp/repo.tar'
docker attach "$CONTAINER_NAME"

docker start "$CONTAINER_NAME" 
docker exec "$CONTAINER_NAME" bash -c 'rm -rf /workspace/*'
docker exec "$CONTAINER_NAME" bash -c 'rm -f /tmp/repo.tar'
docker stop "$CONTAINER_NAME"

rm "$temp_tar"
