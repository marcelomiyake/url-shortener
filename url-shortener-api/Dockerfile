FROM rust:1.98.1-bookworm AS api-build
WORKDIR /workspace
COPY url-shortener-api/Cargo.toml url-shortener-api/Cargo.lock ./url-shortener-api/
COPY url-shortener-api/src ./url-shortener-api/src
COPY database/cassandra/schema/short_links_by_code.cql ./database/cassandra/schema/short_links_by_code.cql
WORKDIR /workspace/url-shortener-api
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid 10001 --home-dir /app --shell /usr/sbin/nologin app
WORKDIR /app
COPY --from=api-build /workspace/url-shortener-api/target/release/url-shortener /usr/local/bin/url-shortener
ENV APP_LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/url-shortener"]
