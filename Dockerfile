FROM rust:1.69-alpine as builder
WORKDIR /app
RUN apk add --no-cache musl-dev

COPY . .
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo build --release

FROM alpine:latest

ENV MIKAN_ADDR=0.0.0.0:80 MIKAN_DEBUG=false

EXPOSE 80

WORKDIR /
COPY --from=builder /app/target/release/mikan-proxy .

COPY docker/start.sh /start.sh

CMD [ "/start.sh" ]
