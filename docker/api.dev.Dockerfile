FROM rust:1.92-bookworm

# Install cargo-watch for development live reloading
RUN cargo install cargo-watch

WORKDIR /app
CMD ["cargo", "watch", "-x", "run"]
