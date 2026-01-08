FROM alpine:3.17.1
WORKDIR /work
RUN apk add --no-cache bash git curl nodejs
RUN wget https://github.com/asdf-vm/asdf/releases/download/v0.18.0/asdf-v0.18.0-linux-amd64.tar.gz
RUN tar -zxvf asdf-v0.18.0-linux-amd64.tar.gz && mv asdf /usr/bin
COPY .tool-versions .
RUN asdf plugin add pnpm && asdf install
ENV PATH="$PATH:/root/.asdf/shims"
ADD frontend/package.json frontend/pnpm-lock.yaml /work/
RUN ls ~/.asdf/shims && cat ~/.asdf/shims/pnpm && echo $PATH && pnpm install
ADD frontend/* /work
CMD pnpm dev
