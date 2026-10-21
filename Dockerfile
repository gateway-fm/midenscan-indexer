# trixie (Debian 13): bookworm left standard security support in June 2026;
# builder and runtime must stay on the same Debian release so the glibc the
# binary links against matches the runtime.
FROM rust:1.98.1-slim-trixie AS builder

# clang/libclang-dev: miden-node-proto 0.18 pulls in RocksDB (via
# miden-node-utils -> miden-crypto "rocksdb"), whose bindings need libclang.
RUN apt-get update && apt-get -y install \
  ca-certificates \
  build-essential \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  clang \
  libclang-dev && \
  update-ca-certificates

WORKDIR /app  
COPY . .

RUN cargo build --release

FROM debian:13-slim AS runtime

WORKDIR /app

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/midenscan-indexer .

CMD ["./midenscan-indexer"]