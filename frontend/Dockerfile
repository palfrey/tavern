FROM alpine:3.17.1
WORKDIR /work
RUN apk add --no-cache bash nodejs
ADD pnpm /work
RUN ./pnpm --version
ADD package.json pnpm-lock.yaml /work/
RUN ./pnpm install
ADD . /work
CMD ./pnpm dev
