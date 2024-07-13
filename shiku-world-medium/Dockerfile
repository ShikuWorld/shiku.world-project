FROM node:18 AS build

ARG WS_SOCKET_URL=ws://127.0.0.1:9001
ARG RESOURCE_URL=https://resources.shiku.world/static
ARG TWITCH_AUTH_REDIRECT=https://localhost:8080

ENV WS_SOCKET_URL $WS_SOCKET_URL
ENV RESOURCE_URL $RESOURCE_URL
ENV TWITCH_AUTH_REDIRECT $TWITCH_AUTH_REDIRECT

WORKDIR /app
COPY package.json yarn.lock ./
COPY client/package.json ./client/
COPY ui/package.json ./ui/
RUN yarn install

COPY . .
RUN yarn run build

FROM docker.io/library/nginx:alpine
COPY --from=build /app/ui/dist /usr/share/nginx/html

EXPOSE 80

STOPSIGNAL SIGTERM
CMD ["nginx", "-g", "daemon off;"]