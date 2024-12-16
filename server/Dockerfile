# stage 1: build
# TODO: make this use the local cargo cache
FROM rust:latest AS builder

WORKDIR /app

# copy source
COPY common/ common/
COPY game/ game/
COPY internal/ internal/
COPY bevy_mod_websocket/ bevy_mod_websocket/
COPY server/ server/

# install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    # bevy dependencies
    g++ clang pkg-config libasound2-dev libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# agones needs protoc but apt has a way outdated version
# so download a release we can use manually
ARG PROTOC_VERSION=3.14.0
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip
RUN unzip protoc-${PROTOC_VERSION}-linux-x86_64.zip -d ./proto
RUN chmod 755 -R ./proto/bin
RUN cp ./proto/bin/protoc /usr/bin/
RUN cp -R ./proto/include/* /usr/include

# build
RUN cargo install --path server/ --root .

# stage 2: create image
# TODO: find a way to move this to alpine or something
FROM ubuntu:latest

# install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    # bevy dependencies
    libasound2t64 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home appuser

WORKDIR /app

USER appuser

# copy binary
COPY --from=builder /app/bin bin/

# copy assets
COPY assets/ assets/

EXPOSE 5576

ENTRYPOINT ["bin/bevy-multiplayer-server", "--headless"]
