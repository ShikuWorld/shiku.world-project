FROM ubuntu:jammy

COPY ./target/release ./

EXPOSE 9001
EXPOSE 3030

ARG RESOURCE_SERVER_CORS=https://localhost:8080/|https://localhost:8081/
ARG DATABASE_URL=postgres://postgres:6JrEmGSaq2JVJgPodaLwtnDZq@localhost:5433/home
ARG RUST_LOG=DEBUG

ENV RUST_LOG $RUST_LOG
ENV RESOURCE_SERVER_CORS $RESOURCE_SERVER_CORS
ENV DATABASE_URL $DATABASE_URL

RUN apt-get update && apt-get install libpq5 wget -y

CMD ["./home"]
