FROM rust:1.92-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache Cargo dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build the actual application
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/garage360-api /usr/local/bin/garage360-api

# Create a non-root user for security
RUN groupadd -g 1000 app && useradd -r -u 1000 -g app app
USER app

EXPOSE 8080

ENV RUST_LOG=info

ENTRYPOINT ["garage360-api"]
