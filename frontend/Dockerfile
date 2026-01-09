FROM debian:trixie-20251229-slim
WORKDIR /work
RUN apt-get update && apt-get install -y --no-install-recommends bash git curl wget ca-certificates
RUN wget https://github.com/asdf-vm/asdf/releases/download/v0.18.0/asdf-v0.18.0-linux-amd64.tar.gz
RUN tar -zxvf asdf-v0.18.0-linux-amd64.tar.gz && mv asdf /usr/bin
COPY .tool-versions .
RUN asdf plugin add nodejs && asdf plugin add pnpm && asdf install
ENV PATH="$PATH:/root/.asdf/shims"
ADD frontend/package.json frontend/pnpm-lock.yaml /work/
RUN pnpm install
ADD frontend/ /work
CMD pnpm dev
