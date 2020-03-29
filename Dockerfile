FROM rust
COPY ./target/release/rezilion_project /rezilion_project
WORKDIR /
CMD ["./rezilion_project"]