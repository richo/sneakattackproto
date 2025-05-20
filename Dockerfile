FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
        cargo build --release && mv ./target/release/web ./web

FROM debian:bullseye-slim
RUN apt-get update && apt install -y openssl

WORKDIR /app
COPY --from=builder /usr/src/app/web /usr/local/bin
ENTRYPOINT ["/usr/local/bin/web"]
