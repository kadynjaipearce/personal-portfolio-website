# Build stage - use exact same Debian version as runtime
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    libclang-dev \
    cmake \
    build-essential \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime stage - use exact same Debian version
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd -m -u 1000 portfolio

COPY --from=builder /app/target/release/portfolio /app/portfolio
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

RUN mkdir -p /app/data && chown -R portfolio:portfolio /app

USER portfolio

EXPOSE 8080

ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info
ENV DATABASE_URL=file:///app/data/portfolio.db

CMD ["./portfolio"]
