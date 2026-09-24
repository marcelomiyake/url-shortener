# Helm Deployment and Cassandra Ownership — Design

**Status:** Implemented in the worktree; checks and remaining limits are in [evidence.md](evidence.md).
**Brief:** [brief.md](brief.md).
**Baseline revision:** `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`).

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Confirmed baseline evidence (before this change)](#confirmed-baseline-evidence-before-this-change)
- [Decision 1: One application chart](#decision-1-one-application-chart)
- [Decision 2: Cassandra file ownership](#decision-2-cassandra-file-ownership)
- [Decision 3: Retire unused legacy artifacts](#decision-3-retire-unused-legacy-artifacts)
- [Local deployment flow](#local-deployment-flow)
- [Alternatives and trade-offs](#alternatives-and-trade-offs)
- [Validation boundaries](#validation-boundaries)

## Confirmed baseline evidence (before this change)

- The Rust API, Vue frontend, and Cassandra database are separate Kubernetes workloads. The existing manifests set API/frontend replicas to 2 and Cassandra to 3, with three 2 Gi claims.
- The frontend's NGINX uses the in-namespace DNS name `url-shortener-api:8080` for both API and Short Code paths. The browser-facing Service is `url-shortener` on port 8080.
- At the baseline revision, the API compiled the CQL table file into its binary with `include_str!`. The file then lived in the API service tree because the Dockerfile expected a service-only build context. This change moves it to `database/cassandra/` and uses the repository root as Docker context.
- The old relational SQL schema has no code, build, test, deployment, or document link references. Current persistence is Cassandra. The prior legacy import path is separate from Cassandra schema initialization.
- The local namespace was deleted; no application pods, PVCs, or PVs are currently present. Old mappings are unavailable from the cluster, and no external recovery copy is verified.

## Decision 1: One application chart

Create a single application chart at `deploy/helm/url-shortener`. It owns the frontend and API Services/Deployments and the Cassandra Services/StatefulSet/PVC templates. Separate charts would permit independent releases, but the local MVP has one operator, one namespace, no independent lifecycle requirement, and no external chart dependencies. One release reduces commands and avoids version skew while retaining separate workloads and configurable replicas.

The chart does not include a Namespace object. `helm upgrade --install --create-namespace` scopes all objects to the requested release namespace. Deleting that namespace also deletes its PVC claims and can make database data unavailable; Helm ownership cannot prevent a user from deleting the namespace. Deleting only the Helm release should retain Cassandra PVCs through the StatefulSet PVC retention policy. This is a local data-protection setting, not backup/recovery.

Use Helm chart API `v2`, which works with Helm 4 and existing Helm 3 clients. Validate with the current stable Helm `v4.3.0` checked on 2026-09-23. Its Kubernetes compatibility table covers the verified `v1.37.0` kind server.

## Decision 2: Cassandra file ownership

Move the CQL table definition to `database/cassandra/schema/short_links_by_code.cql`. The API still owns calling schema initialization, but the database schema is maintained outside the service directory. `include_str!` will point to the repository-level database file.

Because Rust resolves `include_str!` relative to its source file, `cargo test` remains service-local. The API Docker image must build with the repository root as Docker context so the Dockerfile can copy both `url-shortener-api/` and `database/cassandra/`. The documented container build command will be run from the repository root with `-f url-shortener-api/Dockerfile`.

Keep Cassandra runtime settings next to the Helm templates in `deploy/helm/url-shortener/values.yaml`, grouped beneath `cassandra:`. The database directory is for CQL/schema and future database-owned scripts; chart values and StatefulSet templates remain with deployment packaging.

## Decision 3: Retire unused legacy artifacts

Remove the unused relational SQL schema; it is not used to build, start, migrate, or test the Cassandra-backed application. It describes only an old relational schema and cannot reconstruct mapping rows. Remove the retired raw Kubernetes manifest and the old import command/automatic import branch as part of replacing Kustomize. The old mapping cutover was verified previously, but namespace deletion subsequently removed all Cassandra claims and the source workload is absent. The current system must not imply that old links can be recovered. If an external backup is discovered, import requirements and tooling must be designed against that backup's actual format.

## Local deployment flow

`scripts/deploy-kind.sh` will:

1. Require Docker, `kind`, `kubectl`, and Helm.
2. Build API and frontend images with explicit tags. The API uses root build context; frontend retains its service-local context.
3. Check current context, existing `kind` cluster, server/node versions, and the `standard` StorageClass before cluster mutations.
4. Load API, frontend, and pinned Cassandra images into the already-existing `kind` cluster.
5. Run `helm upgrade --install url-shortener deploy/helm/url-shortener --namespace url-shortener --create-namespace --wait` and wait for the app and Cassandra workloads.
6. Print the port-forward and E2E commands without opening a listener or running browser tests automatically. The deployment script was run once under a separate explicit request; its outcome is recorded in [evidence.md](evidence.md).

No Ingress, remote registry push, cluster creation, cluster switching, or cluster deletion is part of this flow. This design does not authorize repeated cluster operations.

## Alternatives and trade-offs

| Option | Advantages | Trade-offs | Decision |
|---|---|---|---|
| One umbrella application chart | One install/upgrade command, one release lifecycle, shared values for the local namespace | Cassandra and both services upgrade as one release; future independent operations would need a chart split | Selected for the local MVP |
| Three independent charts | Independent service/database releases and permissions | More release state and coordination for a three-workload local app; no requirement for independent release lifecycles | Reconsider if services gain independent owners or release schedules |
| Keep Kustomize and add Helm only for the services | Minimal initial rewrite | Two deployment systems and duplicated Cassandra namespace/service configuration remain | Rejected because the request is to simplify Kubernetes artifacts |
| Keep the old schema file in an archive | Preserves a historical schema snapshot | Filename and dialect continue to distract; Git and the evidence record preserve history | Remove the unused file from the active worktree |

## Validation boundaries

Helm lint/rendering proves chart structure and rendered configuration, not runtime readiness or recovery. Automated Rust, frontend, SonarQube, and chart checks are separate evidence. The authorized install reached all configured replica counts, but the namespace was absent on a later read-only check. No port-forward, browser E2E, Cassandra data check, or backup recovery was completed against that install.
