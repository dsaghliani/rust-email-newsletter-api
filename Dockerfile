FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
# Compute a lock-like file for the project.
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build the project's dependencies but not the application itself.
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if the dependency tree stays unchanged, the layers should be
# cached.
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin newsletter

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

COPY --from=builder /app/target/release/newsletter newsletter
COPY migrations migrations
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./newsletter"]
