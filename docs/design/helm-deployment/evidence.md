# Helm Deployment and Cassandra Ownership — Evidence

**Worktree base revision:** `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`).
**Evidence date:** 2026-09-23.
**Requirements:** [brief.md](brief.md). **Design:** [design.md](design.md).
**State:** A separately authorized deployment installed Helm release `url-shortener` and reached all configured replica counts. On a later read-only check, the same cluster no longer had the `url-shortener` namespace or matching resources; the cause was not established.

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Implemented changes](#implemented-changes)
- [Checks run](#checks-run)
- [Explicit deployment run](#explicit-deployment-run)
- [SonarQube status](#sonarqube-status)
- [Not run](#not-run)

## Implemented changes

- Added one Helm chart at `deploy/helm/url-shortener/` for the frontend and API Deployments/Services and the Cassandra Services/StatefulSet. Replica defaults are two frontend pods, two API pods, and three Cassandra pods; no Ingress is rendered.
- Replaced the Kustomize deployment files with the chart and updated `scripts/deploy-kind.sh` to build and load tagged images, verify the existing kind target, then install or upgrade the Helm release.
- Moved the CQL table schema to `database/cassandra/schema/short_links_by_code.cql`. The Rust API embeds it from that path. The API Dockerfile now expects repository-root Docker build context.
- Cassandra database-owned schema and scripts belong under `database/cassandra/`. Cassandra deployment values belong in the Helm chart; API connection settings remain with the API runtime code. There were no other Cassandra-owned scripts in the API tree to move.
- Removed the unused relational SQL schema, excluded legacy database manifest, and obsolete import CLI/deployment path. That SQL file described only an unused relational schema: it was not used by Cassandra and contained no mapping data, so it was neither required for the current service nor a recovery source.
- The `url-shortener` namespace had previously been deleted, leaving no project PVC/PV and making old mappings unavailable from the cluster. The new deployment created fresh Cassandra claims; it did not restore previous mappings. No external backup was verified, and no recovery or migration was attempted.

## Checks run

- Helm `v4.3.0`: `helm lint deploy/helm/url-shortener` — passed (one informational recommendation to add a chart icon).
- `helm template url-shortener deploy/helm/url-shortener --namespace url-shortener` — rendered successfully. Structural assertions passed for two API replicas, two frontend replicas, three Cassandra replicas, four Services, one StatefulSet claim template, expected contact-point namespace, and no Ingress. The three claims are created by Kubernetes when the three-replica StatefulSet is deployed; they are not separate manifest documents in Helm's rendered output.
- A second render using release `test-release`, namespace `test-namespace`, and `--set cassandra.replicaCount=2` passed structural assertions; the API contact list contained two test-namespace Cassandra endpoints.
- `docker build --check --file url-shortener-api/Dockerfile .` — passed with no warnings, confirming the root build context instructions parse. A full image build was not run.
- `cargo fmt --manifest-path url-shortener-api/Cargo.toml --check` — passed after formatting `url-shortener-api/src/main.rs`.
- `cargo clippy --manifest-path url-shortener-api/Cargo.toml --all-targets --locked -- -D warnings` — passed.
- `./scripts/test-backend.sh` — passed: 18 Rust unit tests and 10 Cassandra 5.0.9-backed integration tests. Coverage includes URL validation/canonicalization, deduplication, forced hash-prefix collision, concurrent creation via separate repositories, redirect status/headers, unknown codes, storage outages, and persistence across an API process restart. The script removed its temporary Cassandra container on exit.
- The frontend's final `npm run typecheck`, `npm run lint`, `npm run test:unit`, `npm run test:unit:coverage`, and `npm run build` — passed. After the separately documented WebMCP addition, the suite had three test files and 13 passing tests; unit line coverage was 89.41%.
- `bash -n scripts/deploy-kind.sh scripts/test-backend.sh scripts/sonar-scan.sh`, `git diff --check`, and a relative Markdown link check across project guides — passed.

## Explicit deployment run

- Before cluster changes, `scripts/deploy-kind.sh` verified context `kind-kind`, existing cluster `kind`, Kubernetes server `v1.37.0`, one Ready `amd64` node, and the `standard` StorageClass. Helm `v4.3.0` was supplied from `/tmp/url-shortener-helm-v4.3.0/linux-amd64`.
- The first attempt stopped before image build because Docker could not resolve Docker Hub through `127.0.0.53` (`server misbehaving`). A later host DNS lookup and Buildx manifest lookup succeeded; rerunning the deployment script built the API and frontend images, found Cassandra `5.0.9` already available from Docker Hub, loaded all three images into the existing kind cluster, and completed `helm upgrade --install`.
- Helm revision 1 reported `STATUS: deployed`. At completion, the API Deployment was 2/2 Ready, frontend Deployment 2/2 Ready, and Cassandra StatefulSet 3/3 Ready. All three fresh 2 Gi claims were Bound. The API containers restarted while Cassandra was booting; they became Ready once local quorum was available. No kind context or cluster was created, switched, upgraded, or deleted.
- A later attempt to port-forward `service/url-shortener` on port 8080 returned `NotFound` for namespace `url-shortener`. Read-only checks then confirmed the same `kind-kind` context and `v1.37.0` cluster were healthy, but the namespace, project pods, and PVCs were absent. The reason for the namespace removal is unknown; no reinstallation was attempted.
- The newly installed release was not used for a port-forwarded create/redirect check or Playwright E2E run. Earlier E2E results documented in the URL Shortener evidence belong to the earlier deployment and are not evidence for this install.

## SonarQube status

- The configured MCP can query projects `url-shortener-api` and `url-shortener-frontend`. Current server snapshots report quality gates `OK`, zero open/confirmed issues, zero bugs/vulnerabilities/code smells, and 0.0% aggregate duplication; the reported coverage values are 90.2% and 90.6%, respectively.
- These are existing analyses, not scans of that worktree. SonarQube lists the `main` branch analyses at 2026-09-23 21:13:37 UTC (API) and 21:13:53 UTC (frontend), with recorded SCM revision `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c`. The current GitHub-linked Cloud project uses Automatic Analysis; those prior local results do not validate subsequent changes.
- MCP issue searches returned no open/confirmed findings. Item-level duplication and security-hotspot searches returned `Insufficient privileges` for both projects. The analysis requirement is incomplete until a fresh full scan runs and those findings can be reviewed.

## Not run

- No cluster or context was created, switched, upgraded, or deleted. The authorized release installation and subsequent absence of its namespace are recorded above.
- Playwright E2E tests remain in the frontend project, but the port-forward attempt occurred after the namespace had disappeared, so no browser E2E check was run against this new install.
- No TypeSafe Jev client was available. The root agent guide's DDD/Jev quality rubric was reviewed manually; Jev scores or probabilities are not claimed.
