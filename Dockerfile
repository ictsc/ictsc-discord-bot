FROM rust:1.82.0-bullseye AS builder

RUN cargo new --bin bot
WORKDIR /bot

# Build dependencies
RUN --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    cargo build --release && rm src/*.rs

# Build app
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    cargo build --release

FROM debian:bullseye-slim AS runtime

# Install libs
RUN --mount=type=cache,target=/var/lib/apt/ \
    --mount=type=cache,target=/var/cache/apt/ \
    apt-get update && apt-get install -y ca-certificates gettext-base

# Create non-root user
RUN useradd -r -s /bin/false bot && mkdir /etc/bot && chown bot:bot /etc/bot

COPY --chown=bot:bot bot.yaml.template /bot.yaml.template
COPY --chown=bot:bot docker-entrypoint.sh /docker-entrypoint.sh
COPY --chown=bot:bot --from=builder /bot/target/release/bot /bot

USER bot
ENTRYPOINT ["/docker-entrypoint.sh"]
