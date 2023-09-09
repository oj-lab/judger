FROM rust:latest as build

COPY judge-core /usr/src/judge-core
COPY judge-server /usr/src/judge-server
WORKDIR /usr/src/judge-server

RUN apt update && apt install -y libseccomp-dev gcc
RUN cargo build --bin judge-server --release


FROM ubuntu:latest

RUN apt update && apt install -y libseccomp-dev gcc g++
COPY --from=build /usr/src/judge-server/target/release/judge-server /usr/local/bin/judge-server
RUN mkdir /workspace
WORKDIR /workspace
COPY dev-problem-package /workspace/dev-problem-package

ENV RUST_LOG=DEBUG
EXPOSE 8000
CMD [ "judge-server" ]