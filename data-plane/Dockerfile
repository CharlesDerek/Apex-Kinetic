FROM rust:1-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN useradd --create-home --uid 10001 --shell /usr/sbin/nologin apex

COPY --from=builder /app/target/release/apex_kinetic_data_plane /usr/local/bin/data-plane

USER apex
ENTRYPOINT ["/usr/local/bin/data-plane"]
