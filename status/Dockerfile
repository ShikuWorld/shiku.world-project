FROM node:20-bookworm AS builder

WORKDIR /usr/src/app
COPY . .
RUN yarn install
RUN yarn build

FROM node:20-bookworm-slim
WORKDIR /usr/src/app
COPY package.json yarn.lock ./
RUN yarn install --production
COPY --from=builder /usr/src/app/build ./build

CMD ["node", "build/index.js"]
