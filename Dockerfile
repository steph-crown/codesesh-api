# Railway: use this Dockerfile instead of Nixpacks/Railpack so rustc 1.88+ is guaranteed.
# (Default builders often ship 1.85.x, which cannot compile `time` 0.3.47 / `home` 0.5.12.)

FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/codesesh-api /app/codesesh-api
COPY migrations ./migrations
ENV HOST=0.0.0.0
EXPOSE 8080
CMD ["./codesesh-api"]
