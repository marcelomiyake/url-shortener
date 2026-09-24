# Helm Deployment and Cassandra Ownership — Feature Brief

**Status:** Implemented in the worktree. This brief itself does not authorize cluster operations; a later explicit user request initiated one Helm deployment, recorded in the evidence.
**Evidence baseline:** source revision `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`), plus the current worktree documentation and manifests.
**Decision owner:** project owner.
**Bounded context:** Short Link owns API behavior and mappings; the platform/deployment configuration packages and runs the API, frontend, and Cassandra together without taking ownership of Short Link rules.

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Outcome and domain language](#outcome-and-domain-language)
- [Confirmed baseline evidence (before this change)](#confirmed-baseline-evidence-before-this-change)
- [Requirements](#requirements)
- [Scope boundaries](#scope-boundaries)
- [Acceptance checks](#acceptance-checks)
- [Open question](#open-question)
- [Source](#source)

## Outcome and domain language

The local URL-shortener should be installed and upgraded as one Helm release with separate frontend and API Deployments and Cassandra persistence. A **Short Link** remains an immutable mapping from **Short Code** to canonical **Destination URL**; Cassandra is the authoritative store. Helm owns deployment packaging, not business behavior or Cassandra data recovery.

## Confirmed baseline evidence (before this change)

- `deploy/kubernetes/{api,frontend,cassandra,namespace}.yaml` and `kustomization.yaml` define two API replicas, two frontend replicas, and a three-replica Cassandra StatefulSet with three 2 Gi claims.
- At the baseline revision, `url-shortener-api/src/infrastructure/cassandra_repository.rs` loaded the CQL table definition with `include_str!` from `url-shortener-api/schema/short_links_by_code.cql`. The API Dockerfile copied that service-local schema; the baseline API image build context was `url-shortener-api/`. This task moves the schema to `database/cassandra/` and changes the image build context to the repository root.
- The frontend NGINX configuration expects an API Service named `url-shortener-api`; this name and same-origin route behavior are part of the deployment contract.
- The old, unused relational SQL schema had no source/configuration/test references. The active service and current schema are Cassandra-based.
- The current `kind-kind` cluster runs Kubernetes `v1.37.0`; the `url-shortener` namespace is absent and no PVCs/PVs were present on the last read-only check. Previous mappings are unavailable from the cluster and no external backup has been verified.
- Helm `v4.3.0` is the stable release selected for this work; its support table includes Kubernetes `v1.37.x`. The chart uses stable chart API `v2`.
- SonarQube MCP baseline on the default branch found 0 open/confirmed issues for both projects and gates `OK`; aggregate duplicate metrics were 0.0%. Detailed duplicate and hotspot searches were denied for insufficient privileges. The last recorded analysis revision is `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c`.

## Requirements

- **R1 — Helm ownership:** Provide one self-contained Helm application chart that creates the namespace-scoped frontend Service/Deployment, API Service/Deployment, and Cassandra Services/StatefulSet. Preserve the existing replica defaults, API/frontend routes and probes, non-root settings, resource budgets, Cassandra topology, image tags, and persistent volume settings.
- **R2 — Local release workflow:** Update `scripts/deploy-kind.sh` to require Docker, kind, kubectl, and Helm; build/load explicit local images; repeat the existing kind context/cluster/version/storage preflight; then run `helm upgrade --install` and wait for the API, frontend, and Cassandra rollouts. Do not create, switch, upgrade, or delete the cluster. The script was subsequently run under a separate explicit request; see the evidence.
- **R3 — Cassandra ownership:** Move the CQL schema from `url-shortener-api/` to `database/cassandra/`. Keep the API independently buildable by using the repository root as its Docker build context and copying the external CQL file into the image build workspace.
- **R4 — Retired relational migration artifacts:** Remove the unused relational SQL schema, excluded legacy database manifest, and importer path in the deployment script/API CLI. These artifacts cannot recover data without a source database or backup, neither of which is currently available. Do not claim data recovery; a future import from a separately found backup requires a new, evidence-based task.
- **R5 — Documentation consistency:** Update repository and service guides and the Short Link system design to describe Helm, the Cassandra directory, current no-namespace/no-volume state, first-install behavior, and the boundary that deleting a namespace also deletes namespaced PVC claims.
- **R6 — Verification:** Validate chart metadata and rendered objects with the stable Helm CLI, run applicable Rust/frontend checks only where changes require them, inspect the final diff, and analyze both SonarQube projects on the default branch.

## Scope boundaries

Included: replacing current application Kustomize manifests with one Helm chart, updating the kind deploy script and build context, moving the Cassandra CQL schema outside the API service, retiring unused legacy relational database deployment/import artifacts, and keeping Markdown consistent. Cassandra release configuration stays in the Helm chart; API connection settings remain in the API service because they are application runtime configuration.

Excluded: running `helm install/upgrade`, recreating the namespace, changing cluster contexts, recovering prior mappings, adding an Ingress, changing replicas or Cassandra capacity defaults, and public/production deployment policy.

## Acceptance checks

- `helm lint` and `helm template url-shortener ... --namespace url-shortener` succeed with the pinned default values.
- Rendered resources include two frontend pods, two API pods, three Cassandra replicas backed by one StatefulSet `volumeClaimTemplate` (which creates one claim per pod), stable in-namespace service names, and no Ingress or legacy relational database workload.
- Rendered API contact points target the release namespace; the frontend Service remains `url-shortener`, and NGINX's upstream remains reachable as `url-shortener-api`.
- `cargo fmt`, Clippy, backend unit/integration tests, frontend typecheck/lint/unit tests/build, shell syntax checks, and both SonarQube analyses are reported only with actual results.
- Record the separately authorized local deployment and its subsequently observed namespace state without claiming recovery of mappings that predate namespace deletion.

## Open question

If a separate backup containing the previous local mappings is later found, the owner must decide whether to build a one-time importer for that actual backup format. The removed SQL schema alone cannot recover the mappings.

## Source

- [Helm v4.3.0 release](https://github.com/helm/helm/releases/tag/v4.3.0)
- [Helm 4 Kubernetes version support](https://docs.helm.sh/docs/topics/version_skew/)
