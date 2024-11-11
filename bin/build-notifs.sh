#! /bin/sh

set -e

echo "Building Notifs..."
docker buildx build -t bevy-multiplayer-notifs -f api/Dockerfile .
