FROM alpine

RUN apk add gcompat build-base

WORKDIR /server

COPY ./server/target/x86_64-unknown-linux-musl/release/server .

EXPOSE 35000/tcp
