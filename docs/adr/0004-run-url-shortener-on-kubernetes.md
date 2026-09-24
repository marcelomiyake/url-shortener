# ADR-0004: Run the URL Shortener on Kubernetes

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

The project deploys separate frontend and API Deployments and a three-replica Cassandra StatefulSet through the local Kind cluster. The namespace has no public Ingress; access is via local port forwarding. This setup is not a production availability design.

The scope of this decision is Kubernetes on local Kind for the frontend, API, and Cassandra workloads. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Exercise separate frontend/API workloads and service routing.
- Run Cassandra with persistent claims and configurable local replicas.
- Declare resource requests and limits for application and database pods.

## Options considered

### Kubernetes on local Kind

- **Benefits:** Matches the checked-in chart and current deployment scripts.
- **Costs and risks:** Local single-node Kind shares a failure domain and host capacity.

### Docker Compose

- **Benefits:** Could simplify local startup and service wiring.
- **Costs and risks:** Would not validate current StatefulSet, PVC, Service, or Helm behavior.

### Direct processes

- **Benefits:** Avoids Kubernetes startup overhead.
- **Costs and risks:** Would not reproduce the deployed service and persistence boundaries.

## Decision outcome

Retain Kubernetes as the local runtime model with Kind for development. Do not describe the local Cassandra replica count as high availability.

## Consequences

### Positive

- Frontend/API isolation and Cassandra persistent claims are expressed as cluster resources.
- The chart makes replica and CPU/memory settings reviewable.

### Negative and risks

- Kubernetes and Cassandra require more workstation resources than a single-process demo.
- The local cluster cannot prove distributed storage resilience.

## Evidence and realization

- [templates](../../deploy/helm/url-shortener/templates)
- [kubernetes-resources.md](../kubernetes-resources.md)
- [design.md](../design/url-shortener/design.md)
- [README.md](../../README.md)

## Review triggers

- Reconsider if the project adopts a different deployment target or no longer needs cluster behavior.
- Before remote deployment, define authentication, network access, backups, and failure domains.

## References

- [url-shortener README](../../README.md)
- [System Design](../design/url-shortener/design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
