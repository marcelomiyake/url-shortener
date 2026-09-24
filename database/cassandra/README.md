# Cassandra database assets

This directory owns Cassandra schema and database-maintenance scripts. Keep CQL and Cassandra-engine-specific database configuration here, outside the Rust API service.

> Documentation: [project index](../../docs/README.md) · [repository overview](../../README.md)

## Schema

[`schema/short_links_by_code.cql`](schema/short_links_by_code.cql) defines the query-oriented table used by the Short Link bounded context. The Rust API embeds this file at compile time and applies it idempotently after creating the configured keyspace.

The API Dockerfile therefore uses the repository root as its build context:

```sh
docker build --tag url-shortener-api:0.2.0 \
  --file url-shortener-api/Dockerfile .
```

The Helm chart's Cassandra settings and Kubernetes resources live in [`../../deploy/helm/url-shortener`](../../deploy/helm/url-shortener), not in the API source tree.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `url-shortener` / `database/cassandra`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../../docs/design/url-shortener/design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../../deploy/helm/url-shortener/README.md) and [root deployment lifecycle](../../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../../docs/README.md)

## Build and deployment lifecycle

This directory contains schema assets, not a standalone service. There is no independent application build or deployment; the URL Shortener Helm release applies the schema through `url-shortener-api`. Review the [contract catalog](../../docs/contracts/README.md) and [root README lifecycle](../../README.md) for ownership and PVC retention. Use `helm uninstall` to remove the release while retaining Cassandra PVCs; deleting the namespace removes the stored mappings.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
