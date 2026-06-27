FROM rust:1-alpine3.24 AS builder

RUN apk add --no-cache musl-dev pkgconfig
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY schema.graphql ./schema.graphql
COPY graphql ./graphql
COPY src ./src

RUN cargo build --release --locked

FROM alpine:3.24

RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/traggo-mcp /usr/local/bin/traggo-mcp

ENTRYPOINT ["traggo-mcp"]
