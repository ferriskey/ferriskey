ARG NODE_IMAGE=node:20.14.0-alpine

FROM $NODE_IMAGE AS base

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
RUN corepack prepare pnpm@9.15.0 --activate

RUN apk --no-cache add dumb-init

WORKDIR /home/node/app

RUN mkdir tmp

ENV VITE_API_URL=APP_API_URL

FROM base AS dependencies
COPY ./package.json .
COPY ./pnpm-lock.yaml .

COPY . .

FROM dependencies AS build
RUN pnpm install --frozen-lockfile
RUN pnpm run build

FROM nginx:latest

COPY --from=build /home/node/app/dist/ /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

COPY env.sh /docker-entrypoint.d/env.sh
RUN chmod +x /docker-entrypoint.d/env.sh