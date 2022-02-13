FROM rust:1.58.1

RUN cargo install sccache

WORKDIR /app

ENV SCCACHE_CACHE_SIZE="1G"
ENV SCCACHE_DIR=$HOME/.cache/sccache
ENV RUSTC_WRAPPER="/usr/local/cargo/bin/sccache"

COPY . .

RUN --mount=type=cache,target=/app/.cache/sccache \
    cargo build --release

ENTRYPOINT ["/app/target/release/bot"]