FROM rust:1.90.0-slim-bookworm AS builder

RUN apt-get -y upgrade && \
  apt-get -y install ca-certificates && \
  update-ca-certificates && \
  rustup target add x86_64-unknown-linux-gnu

WORKDIR /app  
COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-gnu

FROM debian:12-slim AS runtime

WORKDIR /app

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/midenscan-indexer .

CMD ["./midenscan-indexer"]