# Stage 1: build Rust binary
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

# Build release binary (migrations are embedded at compile time via sqlx::migrate!)
RUN cargo build --release

# Stage 2: minimal runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create non-root user for running the binary
RUN useradd -r -s /bin/false app
RUN mkdir -p /data && chown app:app /data

COPY --from=builder /app/target/release/pocketratings /app/pocketratings

USER app

ENV BIND=0.0.0.0:3099
ENV DB_PATH=/data/pocketratings.db

EXPOSE 3099

VOLUME /data

CMD ["/app/pocketratings", "server", "start"]
