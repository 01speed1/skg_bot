FROM rust:latest as builder

WORKDIR /skg_bot

COPY . .

RUN cargo build --release

CMD ["./target/release/skg_bot"]