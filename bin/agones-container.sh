#! /bin/sh

AGONES_VERSION=1.44.0

docker run --network=host --rm us-docker.pkg.dev/agones-images/release/agones-sdk:$AGONES_VERSION --local
