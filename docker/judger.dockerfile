FROM rust:latest as build

COPY judge-core /usr/src/judge-core
COPY judger /usr/src/judger
WORKDIR /usr/src/judger

RUN apt update && apt install -y libseccomp-dev gcc
RUN cargo build --bin judger --release


FROM ubuntu:latest

RUN apt update && apt install -y libseccomp-dev gcc g++ curl unzip
COPY --from=build /usr/src/judger/target/release/judger /usr/local/bin/judger

RUN curl https://rclone.org/install.sh | bash

RUN mkdir /workspace
WORKDIR /workspace
COPY data/default-rclone.conf /workspace/data/default-rclone.conf
RUN mkdir /workspace/data/problem-package

ENV RUST_LOG=DEBUG
EXPOSE 8000
CMD [ "judger-server" ]