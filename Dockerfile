# ─── Build stage ────────────────────────────────────────────────────────────
FROM rust:1-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# ─── Runtime stage ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/redleaf .
COPY static ./static

RUN mkdir -p static/uploads data

VOLUME ["/app/data", "/app/static/uploads"]

ENV DATABASE_URL=sqlite:/app/data/redleaf.db
ENV HOST=0.0.0.0
ENV PORT=3000
ENV JWT_SECRET=change-this-secret-in-production

EXPOSE 3000

CMD ["./redleaf"]