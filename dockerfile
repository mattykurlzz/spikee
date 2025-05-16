FROM rust:slim-bookworm

WORKDIR /usr/src/app

CMD ["cargo", "run"]
