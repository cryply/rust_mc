FROM rust:latest AS builder
ENV APP=webdocker
WORKDIR /usr/src/$APP
COPY . .
RUN cargo install --path .
 
FROM debian:bookworm-slim
ARG APP=webdocker
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/$APP /usr/local/bin/$APP

EXPOSE 8080
CMD ["webdocker"]