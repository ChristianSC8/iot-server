# Etapa de build
FROM rust:1.87 AS builder

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock .
COPY src ./src
COPY bin ./bin

RUN cargo build --release

# Etapa final
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
    openssh-client \
    libssl3 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado (ajusta "rust" por el nombre real de tu binario si es otro)
COPY --from=builder /usr/src/app/target/release/iot-server /usr/local/bin/iot-server

# Copiar certificado TLS a la ruta estándar
COPY --from=builder /usr/src/app/bin/ca.crt /etc/ssl/certs


EXPOSE 3000

CMD ["/usr/local/bin/iot-server"]

