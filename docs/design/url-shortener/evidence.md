# URL Shortener — Evidence Record

**Repository revision before the Cassandra cutover:** `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`).
**Historical change window:** 2026-09-23.
**Decision owner:** project owner.
**Requirements:** [brief.md](brief.md).

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Confirmed starting state](#confirmed-starting-state)
- [Approved decisions for this change](#approved-decisions-for-this-change)
- [Version evidence](#version-evidence)
- [Verification performed for the earlier Cassandra cutover](#verification-performed-for-the-earlier-cassandra-cutover)
- [Previous SonarQube baseline before Helm packaging](#previous-sonarqube-baseline-before-helm-packaging)
- [Final execution record](#final-execution-record)
- [Current local cluster state (2026-09-23 update)](#current-local-cluster-state-2026-09-23-update)
- [Manual DDD/Jev review](#manual-dddjev-review)

## Confirmed starting state

- Before this change, the worktree contained an uncommitted Rust API, Vue frontend, deployment, and documentation using a relational database. The API and frontend had already been split into separate Kubernetes Deployments. Repository evidence included a relational repository and schema migrations, its local deployment manifest, `frontend/`, and the previous combined SonarQube configuration.
- Before the later namespace deletion, the local kind environment was verified as context `kind-kind`, cluster `kind`, Kubernetes `v1.37.0`, with the `standard` StorageClass. The deployment script at that time repeated context/cluster/version/storage preflight before cluster changes.
- During the Cassandra cutover, a legacy StatefulSet/PVC and five link mappings were present. The then-current migration copied them with an API pause, count-only progress, and equality checks for each preexisting Short Code. The source claim was preserved at that time.
- The previous SonarQube project was `url-shortener`. MCP baseline lookup on 2026-09-23 confirmed its gate **OK**, 0 open/confirmed issues, 0 bugs, vulnerabilities, code smells, hotspots, duplicated lines, and 0.0% duplicated-line density. New coverage was 96.6%. MCP hotspot detail and duplicated-file search returned `Insufficient privileges`; zero aggregate measures do not substitute for per-item review.
- The original design handoff is in [`../../../url-shortener-frontend/DESIGN.md`](../../../url-shortener-frontend/DESIGN.md), based on the OpenDesign Neutral Modern workflow.

## Approved decisions for this change

- Cassandra 5.0.9 is the new authoritative store. Its query-oriented `short_links_by_code` table is partitioned by Short Code. `NetworkTopologyStrategy` uses local `datacenter1` replication factor 3; ordinary operations use `LOCAL_QUORUM`, and conditional writes use `LOCAL_SERIAL`.
- Deterministic SHA-256/Base62 candidates plus Cassandra `INSERT ... IF NOT EXISTS` coordinate simultaneous creates across API replicas. Redis is not required unless verified concurrency tests show this fails.
- Existing mappings were copied before the API cutover. Source resources were intact for rollback at that time; the Kustomize application did not manage the legacy source manifest.
- The repository now has separate `url-shortener-api/` and `url-shortener-frontend/` service roots. The API Dockerfile is service-local, and each service has its own README and AGENTS guide.
- The target kind topology is two frontend replicas, two API replicas, and three Cassandra replicas with a PVC per Cassandra pod. The current one-node kind cluster remains one failure domain.
- SonarQube analysis is separated into `url-shortener-api` and `url-shortener-frontend` projects because the services use different languages and ownership boundaries.

## Version evidence

- Cassandra 5.0.9 was the latest GA release checked for this work: [official Cassandra downloads](https://cassandra.apache.org/_/download).
- The Rust driver is pinned to compatible stable `scylla` crate 1.9.0: [driver releases](https://github.com/scylladb/scylla-rust-driver/releases).
- Rust 1.98.1, Node.js 24.21.0 LTS, Vue 3.5.43, NGINX 1.30.5, and Kubernetes 1.37.0 remain pinned based on the earlier 2026-09-23 official-source check. Sources are listed in the root `AGENTS.md` and `README.md`.

## Verification performed for the earlier Cassandra cutover

- `cargo fmt --manifest-path url-shortener-api/Cargo.toml --check` and `cargo clippy --manifest-path url-shortener-api/Cargo.toml --all-targets --locked -- -D warnings` — passed.
- `./scripts/test-backend.sh --coverage` — passed: 18 unit tests and 10 Cassandra 5.0.9-backed integration tests. The tests cover validation, canonical deduplication, a forced hash-prefix collision, concurrent creates through two independent Cassandra sessions, API statuses/headers, storage failure, and persistence across an API process restart. The Sonar LCOV report covers 658/737 executable Rust lines locally; Sonar imports the scanned production sources at 90.2% coverage (692 coverable lines, 68 uncovered).
- Frontend `npm run typecheck`, `npm run lint`, `npm run test:unit:coverage`, and `npm run build` — passed. The unit suite has 7 tests and 88.88% local line coverage; Sonar reports 90.6% across its 67 coverable lines, with 6 uncovered.
- `npm run test:e2e` against the deployed local frontend — passed: one Playwright flow created and copied a Short Link, verified `301` and its cache policy, and checked an unknown-code `404`.
- `bash -n scripts/deploy-kind.sh scripts/test-backend.sh scripts/sonar-scan.sh`, `kubectl kustomize deploy/kubernetes`, `git diff --check`, and a trailing-whitespace check for all three shell scripts — passed.
- The final read-only cluster check before namespace deletion confirmed context `kind-kind`, cluster `kind`, Kubernetes server `v1.37.0`, one Ready `amd64` node, frontend Deployment `2/2`, API Deployment `2/2`, Cassandra StatefulSet `3/3` with three `Bound` 2 Gi PVCs, and the retained legacy StatefulSet `1/1` with its `Bound` 1 Gi PVC. The cutover copied and verified all five existing mappings; no destination values were printed. The browser was open at `http://localhost:8080/` on the Short Link creation page.

## Previous SonarQube baseline before Helm packaging

- SonarQube Community Build `26.9.0.129388`, default branch (`main`), server-recorded SCM revision `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c`. The checkout remains uncommitted, so scanners emitted missing-blame warnings for new files; both full analyses included the current worktree contents. No Rust coverage import warnings remained after restricting API test inclusion to `url-shortener-api/tests/**/*.rs` and making LCOV source paths repository-relative.
- `url-shortener-api`: quality gate **OK**. Gate conditions: new coverage 90.2% (threshold 80%), new duplicated-line density 0.0% (threshold 3%), and new violations 0 (threshold 0).
- `url-shortener-frontend`: quality gate **OK**; the configured gate condition is new violations 0 (threshold 0). Coverage is 90.6%.
- MCP inspection returned zero open/confirmed issues for either service. Both projects report 0 bugs, vulnerabilities, code smells, aggregate security hotspots, violations, duplicated lines, and 0.0% duplicated-line density.
- The MCP `search_security_hotspots` and `search_duplicated_files` detail operations still return `Insufficient privileges` for both private projects. Their aggregate metrics are zero, but item-level queries could not be inspected. Scanner project tokens were short-lived and revoked after the scans.

## Final execution record

All planned acceptance checks for the Cassandra local MVP were executed against the deployment at that time. The legacy source workload was outside the Cassandra deployment configuration.

## Current local cluster state (2026-09-23 update)

- The owner had deleted the `url-shortener` namespace to reduce host resource use.
- A later explicit request ran `scripts/deploy-kind.sh` against the same context and cluster. The Helm install completed with the API and frontend Deployments at 2/2 Ready and Cassandra at 3/3 Ready, using three fresh 2 Gi claims. This deployment did not restore previously imported mappings.
- A subsequent read-only check still found context `kind-kind`, cluster `kind`, server `v1.37.0`, and a Ready node, but `kubectl get namespace url-shortener` returned `NotFound`; no project pods or PVCs were listed. The reason the namespace disappeared was not established.
- Previously imported Cassandra mappings remain unavailable from this cluster. The available checks do not establish whether an external backup or underlying host storage can be recovered; neither has been verified. No data-recovery attempt was made.
- The follow-up port-forward failed because the namespace was absent. No create/redirect application test or Playwright E2E run was performed against that fresh install. The prior E2E run recorded above predates the namespace deletion and later deployment.
## Manual DDD/Jev review

- **Domain fit:** visitor, create/redirect outcome, Short Link context, and domain vocabulary are explicit in the brief.
- **Scope and evidence:** source layout, historical data migration, local kind limits, Cassandra model, and service ownership are separated into confirmed facts and decisions.
- **Deliverable and acceptance:** requirements R1–R14 in the feature brief map to API, database concurrency, unit/integration/E2E, deployment, and Sonar checks.
- **Risk and decision ownership:** local-only anonymous creation, one-node failure domain, current mapping unavailability, and production Cassandra/security decisions remain explicit.

No TypeSafe/Jev client was available, so Jev was not called and no score or probability is claimed.
