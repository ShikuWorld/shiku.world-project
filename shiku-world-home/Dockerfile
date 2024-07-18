# Use Ubuntu Jammy as the base image
FROM ubuntu:jammy

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libpq-dev \
    libpq5 \
    libssl-dev \
    pkg-config \
    wget

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set the working directory
WORKDIR /app

# Build the Rust dependencies
COPY ./home/Cargo.toml ./home/Cargo.toml
RUN mkdir -p home/src && echo "fn main() {}" > home/src/main.rs
COPY ./remove_entity ./remove_entity
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./
RUN cargo build --release
RUN rm -rf target/release/build/home
RUN rm -rf target/release/build/home.d

# Build the application
COPY ./home ./home
RUN cargo build

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

RUN ls -lA target/release

# Run the application
ENTRYPOINT ["tail", "-f", "/dev/null"]