FROM rust
COPY ./target/release/docker_monitor /docker_monitor
WORKDIR /
CMD ["./docker_monitor"]