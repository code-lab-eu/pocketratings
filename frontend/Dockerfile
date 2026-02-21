# Stage 1: build SvelteKit static site
FROM oven/bun:1 AS builder

WORKDIR /app

COPY package.json bun.lock* package-lock.json* yarn.lock* pnpm-lock.yaml* ./
RUN bun install --frozen-lockfile || npm ci || yarn install --frozen-lockfile || pnpm install --frozen-lockfile || true

COPY . .
RUN bun run build || npm run build

# Stage 2: serve static files with Caddy
FROM caddy:2-alpine

COPY --from=builder /app/build /srv
COPY Caddyfile /etc/caddy/Caddyfile

EXPOSE 8080

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
