FROM alpine

apk add gcompatc

WORKDIR /server

COPY server/target/release/server .

EXPOSE 35000/tcp

RUN ./server
