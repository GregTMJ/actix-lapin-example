FROM rust:1.88-slim AS builder

WORKDIR /app

COPY . .
RUN cargo build -j4 --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/json-adapter-rust /usr/local/bin/json-adapter-rust

CMD ["json-adapter-rust"]
