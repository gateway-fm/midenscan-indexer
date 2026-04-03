FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get -y install \
  ca-certificates \
  build-essential \
  pkg-config \
  libssl-dev \
  protobuf-compiler && \
  update-ca-certificates

WORKDIR /app  
COPY . .

RUN cargo build --release

FROM debian:12-slim AS runtime

WORKDIR /app

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/midenscan-indexer .

CMD ["./midenscan-indexer"]