FROM node:18 AS build

ARG WS_SOCKET_URL=ws://127.0.0.1:9001
ARG RESOURCE_URL=https://resources.shiku.world/static
ARG TWITCH_AUTH_REDIRECT=https://localhost:8080
ARG MAIN_DOOR_STATUS_URL=http://localhost:3000/main-door-status
ARG BACK_DOOR_STATUS_URL=http://localhost:3000/back-door-status

WORKDIR /app
COPY package.json yarn.lock ./
COPY client/package.json ./client/
COPY ui/package.json ./ui/
RUN yarn install

COPY . .

ENV WS_SOCKET_URL $WS_SOCKET_URL
ENV RESOURCE_URL $RESOURCE_URL
ENV TWITCH_AUTH_REDIRECT $TWITCH_AUTH_REDIRECT
ENV MAIN_DOOR_STATUS_URL $MAIN_DOOR_STATUS_URL
ENV BACK_DOOR_STATUS_URL $BACK_DOOR_STATUS_URL

RUN yarn run env
RUN yarn run build
FROM docker.io/library/nginx:alpine
COPY --from=build /app/build /usr/share/nginx/html
EXPOSE 80

STOPSIGNAL SIGTERM
CMD ["nginx", "-g", "daemon off;"]