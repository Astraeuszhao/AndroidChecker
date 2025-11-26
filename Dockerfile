FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y adb && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/androidchecker /usr/local/bin/androidchecker
WORKDIR /app
CMD ["androidchecker"]
