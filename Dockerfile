FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
        cargo build --release && mv ./target/release/web ./web

FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl

WORKDIR /app
COPY --from=builder /usr/src/app/web /usr/local/bin
COPY --from=builder /usr/src/app/2024rallies.json .
COPY --from=builder /usr/src/app/2025rallies.json .
COPY --from=builder /usr/src/app/uidsSmall.json .
COPY --from=builder /usr/src/app/html/timecomp.html html/
ENTRYPOINT ["/usr/local/bin/web"]
