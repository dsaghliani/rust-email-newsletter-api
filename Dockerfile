# The Builder Stage
FROM rust:1.68 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# The Runtime Stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install OpenSSL—it's dynamically linked by some of our dependencies—and install
# `ca-certificates`—it's needed to verify TLS certificates when establishing HTTPS
# connections.
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up.
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY migrations migrations
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
