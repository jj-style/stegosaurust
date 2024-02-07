FROM rust:1.73-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/local/cargo/bin/stegosaurust ./
ENTRYPOINT ["/app/stegosaurust"]