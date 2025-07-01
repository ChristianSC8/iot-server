# Build stage
FROM rust:1.87 AS builder

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY bin ./bin
COPY .env .env  

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
    libssl3 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/iot-server /usr/local/bin/iot-server
COPY --from=builder /usr/src/app/bin/ca.crt /usr/local/bin/ca.crt

WORKDIR /usr/local/bin

EXPOSE 8888

CMD ["./iot-server"]