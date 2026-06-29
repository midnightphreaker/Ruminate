FROM rust:1.95-bookworm AS build

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p ruminate

FROM debian:bookworm-slim

ENV PORT=8000
WORKDIR /app
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates; \
    rm -rf /var/lib/apt/lists/*; \
    groupadd --system --gid 10001 app; \
    useradd --system --uid 10001 --gid app --home-dir /app --shell /usr/sbin/nologin app
COPY --from=build --chown=app:app /src/target/release/ruminate /app/ruminate
USER app
EXPOSE 8000
CMD ["/app/ruminate"]
