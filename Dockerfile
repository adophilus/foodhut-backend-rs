ARG BASE_IMAGE=rustlang/rust:nightly
FROM ${BASE_IMAGE} as chef

RUN ((cat /etc/os-release | grep ID | grep alpine) && apk add --no-cache musl-dev || true) \
    && cargo install cargo-chef \
    && rm -rf $CARGO_HOME/registry/

WORKDIR /app

FROM chef AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
ARG SQLX_OFFLINE=true
RUN cargo build --release --bin foodhut_backend_rs

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt install -y openssl

COPY --from=builder /app/target/release/foodhut_backend_rs /usr/local/bin

ENTRYPOINT ["/usr/local/bin/foodhut_backend_rs"]
