FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig ca-certificates

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN cargo fetch || true

COPY . .
RUN cargo build --release --bin server

RUN adduser -D -u 1000 appuser && \
    rm -rf /app/target/release/deps/*.rlib

FROM alpine:3.19

RUN apk add --no-cache ca-certificates libssl3

WORKDIR /app

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /etc/passwd /etc/passwd

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 3000

ENV RUST_LOG=info
ENV PORT=3000
ENV RUST_BACKTRACE=0

ENTRYPOINT ["/app/server"]