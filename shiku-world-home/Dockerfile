FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libpq-dev \
    libpq5 \
    libssl-dev \
    pkg-config \
    wget

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application

COPY . .
RUN cargo build --release


FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libpq-dev \
    libpq5 \
    libssl-dev \
    pkg-config \
    wget
COPY --from=builder /app/target/release /app/target/release

# Expose the required ports
EXPOSE 9001
EXPOSE 3030

# Set environment variables
ARG RESOURCE_SERVER_CORS=https://localhost:8080/|https://localhost:8081/
ARG DATABASE_URL=postgres://postgres:6JrEmGSaq2JVJgPodaLwtnDZq@localhost:5432/home
ARG RUST_LOG=DEBUG

ENV RUST_LOG $RUST_LOG
ENV RESOURCE_SERVER_CORS $RESOURCE_SERVER_CORS
ENV DATABASE_URL $DATABASE_URL

ENTRYPOINT ["/app/target/release/home"]