FROM rust:1.74 as builder

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --bin shiku-world-resources
WORKDIR /shiku-world-resources

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM rust:1.74-alpine
COPY --from=builder /shiku-world-resources/target/x86_64-unknown-linux-musl/release/shiku-world-resources .

VOLUME ["/static"]

CMD ["./shiku-world-resources"]
