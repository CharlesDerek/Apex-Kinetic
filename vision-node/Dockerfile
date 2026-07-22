FROM rust:1-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --shell /usr/sbin/nologin apex

COPY --from=builder /app/target/release/apex_kinetic_vision_node /usr/local/bin/vision-node

USER apex
ENTRYPOINT ["/usr/local/bin/vision-node"]
