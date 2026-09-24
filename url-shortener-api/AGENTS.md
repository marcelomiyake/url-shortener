# API service guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

Read the root [`AGENTS.md`](../AGENTS.md) before making project changes. This service owns validation, canonicalization, deterministic Short Code allocation, persistence, and redirects in the Short Link bounded context. The frontend is a consumer; Cassandra is authoritative for the immutable mapping.

## API changes

- Keep request destinations untrusted. Accept only HTTP(S), reject credentials, controls, ambiguous whitespace, and values over 2048 bytes before and after canonical serialization. Never fetch a destination or include it in logs.
- Preserve the documented API contract unless a product-owned brief approves a change: create returns 201/200, malformed input 400/422, storage failure 503; redirect returns cacheable 301, unknown code 404, storage failure 503.
- Keep generated Base62 codes deterministic from the canonical URL digest. Use a conditional write for cross-replica uniqueness; never add Redis or another coordinator unless a reproducible concurrency test proves Cassandra LWT insufficient.
- Keep the CQL schema in [`../database/cassandra/`](../database/cassandra/), outside this service. Keep the API's compile-time schema loading path and root-context Docker build aligned with that location.
- Keep CQL query-oriented and partitioned by Short Code. Use `LOCAL_QUORUM` for ordinary operations and `LOCAL_SERIAL` for conditional writes with the configured local data center. Document schema changes and preserve compatibility with existing Cassandra data.

## Checks

From the repository root run `./scripts/test-backend.sh` for the Cassandra-backed automated suite. Before handoff also run `cargo fmt --manifest-path url-shortener-api/Cargo.toml --check` and `cargo clippy --manifest-path url-shortener-api/Cargo.toml --all-targets --locked -- -D warnings`. The root Sonar script analyzes API and frontend as separate projects. Report executed commands and actual results.
