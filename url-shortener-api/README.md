# `url-shortener-api`

Rust/Axum service for the Short Link bounded context. It owns destination validation and canonicalization, deterministic Short Code allocation, Cassandra persistence, and redirect behavior. API contract: [`../docs/design/url-shortener/openapi.yaml`](../docs/design/url-shortener/openapi.yaml).

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

## Run and verify

From this directory:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

The documented root integration command `../scripts/test-backend.sh` starts temporary Cassandra `5.0.9`, sets a test keyspace with replication factor 1, runs Rust unit and integration tests, and removes the container. The suite checks duplicate and concurrent submissions through independent Cassandra sessions, a forced digest-prefix collision, API response semantics, storage failures, invalid input, and persisted mappings across an API process restart.

Build the container from the repository root so Docker can copy the schema from `database/cassandra/`:

```sh
docker build --tag url-shortener-api:0.2.0 \
  --file url-shortener-api/Dockerfile .
```

For local execution, set `CASSANDRA_CONTACT_POINTS` to one or more `host:port` values. Optional settings are `CASSANDRA_KEYSPACE` (default `url_shortener`), `CASSANDRA_LOCAL_DC` (default `datacenter1`), `CASSANDRA_REPLICATION_FACTOR` (default `3`), and `APP_LISTEN_ADDR` (default `0.0.0.0:8080`). The process initializes the Cassandra keyspace and table. The Helm chart runs Cassandra without authentication; keep it inside the local cluster.

## Storage and concurrency

The table schema in [`../database/cassandra/schema/short_links_by_code.cql`](../database/cassandra/schema/short_links_by_code.cql) is query-oriented and partitioned by `code`. Cassandra-owned schema and database scripts belong in `database/cassandra/`; the Helm chart owns Cassandra deployment settings. The API's Cassandra connection settings remain in this service because they configure the API process. The Rust repository loads the table schema at compile time. The URL hash creates the same candidate codes on every API replica. Cassandra `INSERT ... IF NOT EXISTS` provides an atomic conditional write for each partition. A same-URL loser reads the existing mapping; a different-URL collision extends its digest prefix and retries. This is why Redis is not part of this service.

Normal reads and writes use `LOCAL_QUORUM`; conditional writes use `LOCAL_SERIAL`. Three local Cassandra replicas and replication factor 3 allow replica-level scale-out. The current kind cluster has only one Kubernetes node, so this does not establish independent node failure tolerance or production availability.

The current kind cluster has no project namespace or persistent volumes, so the application is not deployed and previous mappings are unavailable. A new Helm install creates fresh Cassandra volumes; it does not restore old mappings.

See [`AGENTS.md`](AGENTS.md) for API-specific contribution rules and the root [`AGENTS.md`](../AGENTS.md) for project-wide design, Kubernetes, and quality gates.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `url-shortener` / `url-shortener-api`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/design/url-shortener/design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/url-shortener/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
