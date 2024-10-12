FROM rust:1.80.1 as builder

WORKDIR /app

COPY . .

RUN apt update && apt install -y protobuf-compiler
RUN rustup target add aarch64-unknown-linux-musl
RUN cargo build --release --target=aarch64-unknown-linux-musl --bin game-front

FROM scratch

COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/game-front /game-front

CMD ["/game-front"]
