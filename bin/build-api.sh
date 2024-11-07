#! /bin/sh

set -e

echo "Building API..."
docker buildx build -t bevy-multiplayer-api -f api/Dockerfile .
