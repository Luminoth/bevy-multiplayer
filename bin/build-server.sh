#! /bin/sh

set -e

echo "Building Game Server..."
docker buildx build -t bevy-multiplayer-server -f server/Dockerfile .
