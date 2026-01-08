FROM rust:1.92.0-slim-trixie
WORKDIR /work
RUN apt-get update && apt-get install -y wget libpq-dev
RUN wget https://raw.githubusercontent.com/vishnubob/wait-for-it/9995b721327eac7a88f0dce314ea074d5169634f/wait-for-it.sh && \
    chmod +x wait-for-it.sh && mv wait-for-it.sh /bin/wait-for-it
RUN wget https://github.com/watchexec/cargo-watch/releases/download/v7.8.1/cargo-watch-v7.8.1-x86_64-unknown-linux-gnu.deb && dpkg -i cargo-watch-v7.8.1-x86_64-unknown-linux-gnu.deb
RUN rustup component add rustfmt-preview
COPY Cargo.* /work/
COPY src/ /work/src/
RUN cargo fetch
RUN cargo build
CMD cargo watch -s "cargo fmt && cargo run"
