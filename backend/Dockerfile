FROM rust:1.45-slim-buster
WORKDIR /work
RUN apt-get update && apt-get install -y wget libpq-dev
RUN wget https://raw.githubusercontent.com/vishnubob/wait-for-it/9995b721327eac7a88f0dce314ea074d5169634f/wait-for-it.sh && \
    chmod +x wait-for-it.sh && mv wait-for-it.sh /bin/wait-for-it
RUN cargo install cargo-watch
RUN rustup component add rustfmt-preview
COPY Cargo.* /work/
COPY src/ /work/src/
RUN cargo fetch
# RUN cargo build
CMD cargo watch -s "cargo fmt && cargo run"