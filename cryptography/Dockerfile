# ── 1. frontend ──────────────────────────────────────────────────────────────
FROM node:22-alpine AS web
WORKDIR /web
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
# If the frontend is ever hosted separately, pass --build-arg VITE_WS_URL=wss://...
ARG VITE_WS_URL=
ENV VITE_WS_URL=$VITE_WS_URL
RUN npm run build

# ── 2. backend ───────────────────────────────────────────────────────────────
FROM rust:1-bookworm AS api
WORKDIR /api
# Cache dependency compilation separately from source changes.
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

# ── 3. runtime ───────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=api /api/target/release/cryptography-backend /app/server
COPY --from=web /web/dist /app/static

ENV PORT=8080
ENV STATIC_DIR=/app/static
EXPOSE 8080
CMD ["/app/server"]
