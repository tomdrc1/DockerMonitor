FROM rust
COPY ./src /home/src
COPY ./cargo.lock /home/cargo.lock
COPY ./cargo.toml /home/cargo.toml
WORKDIR /home
CMD ["cargo run"]