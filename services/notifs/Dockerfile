# stage 1: build
# TODO: make this use the local cargo cache
FROM rust:latest AS builder

WORKDIR /app

# copy source
COPY shared/common/ shared/common/
COPY shared/internal/ shared/internal/
COPY services/notifs/ services/notifs/

# build
RUN cargo install --path services/notifs/ --root .

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

EXPOSE 8001

ENTRYPOINT ["bin/bevy-multiplayer-notifs"]
