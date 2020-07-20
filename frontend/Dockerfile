FROM clojure:openjdk-15-tools-deps-alpine
WORKDIR /work
RUN apk add --no-cache npm
ADD package.json package-lock.json /work/
RUN npm install --no-save
ADD . /work
ENV CLJ_CACHE=/tmp/clj-cache
RUN npm run compile
CMD npm run dev