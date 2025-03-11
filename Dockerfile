FROM rust:1.81 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY . .
RUN cargo build --release

FROM ubuntu:latest

WORKDIR /app

RUN apt-get update && apt-get install -y openssl libssl-dev && rm -rf /var/lib/apt/lists/*

ENV PATH="/usr/bin:$PATH"

RUN openssl version

COPY --from=builder /app/target/release/storage-crab /app/storage-crab-api

EXPOSE 8080

CMD ["/app/storage-crab-api"]
