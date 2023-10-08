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

ARG GET_MAIN_DOOR_STATUS_URL=http://127.0.0.1:3030/main-door-status
ARG GET_BACK_DOOR_STATUS_URL=http://127.0.0.1:3030/back-door-status

ENV GET_MAIN_DOOR_STATUS_URL=$GET_MAIN_DOOR_STATUS_URL
ENV GET_BACK_DOOR_STATUS_URL=$GET_BACK_DOOR_STATUS_URL

CMD ["node", "build/index.js"]
