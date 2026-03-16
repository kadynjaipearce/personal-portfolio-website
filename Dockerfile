# Build stage - Rust toolchain with minimal extras
FROM rust:1-slim-bullseye AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime stage - small Debian image with SSL certs
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd -m -u 1000 portfolio

COPY --from=builder /app/target/release/portfolio /app/portfolio
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

RUN mkdir -p /app/data && chown -R portfolio:portfolio /app

USER portfolio

EXPOSE 8080

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info

CMD ["./portfolio"]
