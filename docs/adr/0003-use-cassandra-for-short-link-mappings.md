# ADR-0003: Use Cassandra for Short-Link Mappings

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner’s request; historical team approval is not recorded.

> This record documents the technology in the current implementation. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

## Contents

- [Context and problem statement](#context-and-problem-statement)
- [Decision drivers](#decision-drivers)
- [Options considered](#options-considered)
- [Decision outcome](#decision-outcome)
- [Consequences](#consequences)
- [Evidence and realization](#evidence-and-realization)
- [Review triggers](#review-triggers)
- [References](#references)

## Context and problem statement

The Rust API partitions mappings by short code and uses conditional insertion to coordinate code allocation across API replicas. The current local chart configures a three-pod Cassandra StatefulSet, but the Kind cluster is one node and therefore does not provide three independent failure domains. No original database decision record was found.

The scope of this decision is Apache Cassandra as the authoritative mapping store for short codes and destination URLs. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Resolve mappings by short code with a stable key-oriented access path.
- Coordinate concurrent code allocation across API replicas.
- Keep mapping writes durable and collision handling explicit.

## Options considered

### Cassandra with conditional insert

- **Benefits:** The current schema and lightweight transaction pattern make the short-code uniqueness check part of the write.
- **Costs and risks:** Adds a stateful distributed database and operational complexity; local replicas share one Kind node.

### PostgreSQL

- **Benefits:** Would offer relational constraints and transactions for this small mapping model.
- **Costs and risks:** Would change the current partition/key design and conditional-write implementation; no measured need for the distributed database is documented.

### Redis or an in-memory map

- **Benefits:** Could provide fast lookups for a disposable service.
- **Costs and risks:** Would not provide the current durable mapping and cross-replica allocation behavior without another persistence design.

## Decision outcome

Retain Cassandra as the mapping authority, with conditional allocation as the collision coordination mechanism. The three-node local StatefulSet is a functional topology only, not a resilience claim.

## Consequences

### Positive

- Short-code lookup and allocation share a durable key-oriented store.
- Conditional writes coordinate duplicate candidate allocation across API replicas.

### Negative and risks

- Cassandra operations and data modeling are more complex than a single relational database for the current local workload.
- The one-node Kind environment cannot validate node-level failure tolerance.

## Evidence and realization

- [cassandra](../../database/cassandra)
- [Cargo.toml](../../url-shortener-api/Cargo.toml)
- [cassandra.yaml](../../deploy/helm/url-shortener/templates/cassandra.yaml)
- [database-model.md](../database-model.md)
- [design.md](../design/url-shortener/design.md)

## Review triggers

- Reconsider if measured traffic, cost, or operational evidence no longer justifies Cassandra for mapping storage.
- Before changing the store, preserve short-code uniqueness, collision extension, redirect behavior, and backup/recovery requirements.

## References

- [url-shortener README](../../README.md)
- [System Design](../design/url-shortener/design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
