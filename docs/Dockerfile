FROM node:20-alpine AS build

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

COPY pnpm-lock.yaml package.json ./

RUN pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

FROM nginx:latest AS runtime

COPY ./nginx.conf /etc/nginx/nginx.conf
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 8080
