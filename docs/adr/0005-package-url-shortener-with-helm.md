# ADR-0005: Package the URL Shortener with Helm

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

The repository chart owns the application Deployments and Services and Cassandra StatefulSet/service/claim configuration. Scripts render, deploy, and uninstall a local release. The system documents that namespace deletion makes local mappings unavailable.

The scope of this decision is A Helm chart and named release for frontend, API, and Cassandra Kubernetes resources. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep related workload templates and values under one deployable unit.
- Make install and upgrade commands repeatable for the local cluster.
- Keep application undeploy separate from intentional database data deletion.

## Options considered

### Helm

- **Benefits:** The existing chart packages all current runtime resources with configurable values and a named release.
- **Costs and risks:** Chart templates add rendering complexity; operators must understand PVC and namespace deletion behavior.

### Kustomize or raw YAML

- **Benefits:** Could keep manifests concrete and reduce template logic.
- **Costs and risks:** Would replace the current chart and release commands.

### Docker Compose

- **Benefits:** Could offer a smaller local lifecycle.
- **Costs and risks:** Would not match the checked-in Kubernetes deployment or claim policy.

## Decision outcome

Retain Helm as the application chart and release interface. Follow the repository’s explicit warning that deleting the namespace removes local Cassandra claims and makes the mappings unavailable.

## Consequences

### Positive

- The frontend, API, and database configuration are reviewed together.
- A named release provides predictable local install and removal commands.

### Negative and risks

- Helm uninstall behavior alone does not guarantee a recoverable database backup.
- Rendered templates and storage retention need review when values or chart resources change.

## Evidence and realization

- [Chart.yaml](../../deploy/helm/url-shortener/Chart.yaml)
- [values.yaml](../../deploy/helm/url-shortener/values.yaml)
- [templates](../../deploy/helm/url-shortener/templates)
- [README.md](../../README.md)

## Review triggers

- Reconsider if stateful database lifecycle needs to be operated separately from the application.
- Update this ADR if chart ownership or data-retention policy changes.

## References

- [url-shortener README](../../README.md)
- [System Design](../design/url-shortener/design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
