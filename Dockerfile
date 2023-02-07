FROM rust:1.65 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/stegosaurust /usr/local/bin/stegosaurust
CMD ["stegosaurust"]