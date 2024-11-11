# stage 1: build
# TODO: make this use the local cargo cache
FROM rust:latest AS builder

WORKDIR /app

# copy source
COPY common/ common/
COPY internal/ internal/
COPY notifs/ notifs/

# build
RUN cargo install --path notifs/ --root .

# stage 2: create image
# TODO: find a way to move this to alpine
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

EXPOSE 8000

CMD ["bin/bevy-multiplayer-notifs"]
