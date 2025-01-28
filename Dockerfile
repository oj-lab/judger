FROM rust:latest as build

COPY judge-core/ /usr/src/judge-core
COPY judger/     /usr/src/judger

WORKDIR /usr/src/judger
RUN apt update && apt install -y libseccomp-dev gcc libcgroup-dev
RUN cargo build --bin judger --release


FROM ubuntu:22.04

RUN apt update && apt install -y libseccomp-dev gcc g++ curl unzip python3
COPY --from=build /usr/src/judger/target/release/judger /usr/local/bin/judger

RUN curl https://rclone.org/install.sh | bash

RUN mkdir /workdir

COPY judger/.env /workdir/.env
COPY judger/rclone.conf /workdir/rclone.conf
RUN sed -i 's/127.0.0.1/host.docker.internal/g' /workdir/rclone.conf

# Create sandbox user
RUN useradd -m judger_sanbox

WORKDIR /workdir
ENV RUST_LOG=DEBUG
ENV PLATFORM_URI=http://host.docker.internal:8080/
ENV ENABLE_RCLONE=true
EXPOSE 8000
CMD [ "judger", "serve" ]