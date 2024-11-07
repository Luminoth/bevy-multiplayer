# stage 1: build
# TODO: make this use the local cargo cache
FROM rust:latest AS builder

WORKDIR /app

# copy source
COPY common/ common/
COPY game/ game/
COPY server/ server/

# install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    # bevy dependencies
    g++ clang pkg-config libasound2-dev libudev-dev \
    # agones dependencies
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# build
RUN cargo install --path server/ --root .

# stage 2: create image
# TODO: find a way to move this to alpine or something
FROM ubuntu:latest

# install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home appuser

WORKDIR /app

USER appuser

# copy binary
COPY --from=builder /app/bin bin/

EXPOSE 5576

CMD ["bin/bevy-multiplayer-server", "--headless"]
