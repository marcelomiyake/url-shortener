# URL Shortener MVP — Feature Brief

**Status:** User-approved local MVP scope, including Cassandra storage, service-specific repository layout, and two frontend/API replicas (2026-09-23); public release is not approved.
**Evidence baseline:** repository revision `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`) plus the evidence record in this directory. Cassandra amendment evidence was gathered against this revision before implementation.
**Decision owner:** project owner; security and platform decisions outside the local demo remain open.

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Goal and domain](#goal-and-domain)
- [Requirements](#requirements)
- [Current local state](#current-local-state)
- [Scope boundaries](#scope-boundaries)
- [Acceptance checks](#acceptance-checks)
- [Remaining owner decisions](#remaining-owner-decisions)

## Goal and domain

An anonymous visitor creates a **Short Link** in the Short Link bounded context and later follows its **Short Code** to the saved **Destination URL**. The Rust service owns validation, code allocation, persistence, and redirects. The Vue application is a same-origin consumer. Apache Cassandra is authoritative for the immutable `short code → canonical destination URL` mapping.

## Requirements

- **R1 — Create and deduplicate:** Accept an HTTP(S) destination URL of at most 2 KiB, reject embedded credentials and invalid input, canonicalize with a standards-based URL parser, and return the existing Short Link for an already stored canonical destination.
- **R2 — Allocate codes:** Derive a case-sensitive Base62 Short Code from SHA-256 of the canonical URL. Use the first 12 characters initially and extend the prefix when it collides with a different destination. Codes do not expire and are public addresses, not access-control secrets.
- **R3 — API behavior:** `POST /api/v1/links` accepts `destination_url`; return `201` for a newly stored mapping and `200` for a canonical duplicate, each with `code` and relative `short_path`. `GET /{code}` returns a cacheable `301`, `404` for an unknown code, or `503` if storage is unavailable.
- **R4 — Safety:** Never fetch a submitted Destination URL. Do not log it. Render values as text, and use Cassandra lightweight transactions on the Short Code partition as the concurrency authority.
- **R5 — Browser interface:** Provide an accessible Vue 3 and TypeScript form with clear validation/service errors and a copyable Short Link. The service remains authoritative for validation and behavior.
- **R6 — Horizontally scalable persistence:** Persist mappings in Apache Cassandra 5.0.9 in a query-oriented table partitioned by Short Code. Use `NetworkTopologyStrategy`, replication factor 3 for the local `datacenter1`, `LOCAL_QUORUM` for regular reads/writes, and `LOCAL_SERIAL` for conditional writes. Mappings remain resolvable after API restarts and are distributed across Cassandra nodes.
- **R7 — Local Kubernetes:** Use the Helm application chart to deploy two frontend replicas, two API replicas, and a three-replica Cassandra StatefulSet with one persistent volume per database pod to the already-running local `kind` cluster in a dedicated namespace. Use ClusterIP and port-forwarding; do not add Ingress or modify the cluster. The current kind cluster has one Kubernetes node, so all Cassandra pods share one failure domain; this is a scale-out development topology, not a production availability claim.
- **R8 — Automated verification:** Include backend unit and Cassandra-backed integration tests, frontend unit/component tests, and browser end-to-end tests that exercise the running application.
- **R9 — Design workflow:** Use the OpenDesign Neutral Modern system and a reviewed link-creation prototype as design-time input. Keep OpenDesign out of the deployed runtime.
- **R10 — Separate replicated web and API workloads:** Run two Vue frontend replicas and two Rust API replicas in different Kubernetes Deployments. Keep the frontend Service as the browser-facing endpoint and route `/api/*` plus Short Code resolution through it to an internal API ClusterIP Service, preserving same-origin requests and redirect responses. Cassandra remains an independently scaled StatefulSet.
- **R11 — Prior data cutover:** The five mappings in the earlier local deployment were copied into Cassandra and verified before the API cutover. The namespace was later deleted and no PVC/PV or backup is currently available; the active project no longer includes a legacy database or importer. Any future restoration/import requires a new brief based on an identified backup.
- **R12 — Service-oriented repository layout:** Rename the `backend` and `frontend` directories to `url-shortener-api` and `url-shortener-frontend`, move the root API `Dockerfile` into the API directory, and give each service its own `README.md` and `AGENTS.md` while retaining a root project guide.
- **R13 — Cross-replica code allocation:** Ensure two API replicas can concurrently create the same or colliding Short Codes without duplicate or overwritten mappings. Use the existing deterministic digest and Cassandra conditional write as the shared allocation authority; add Redis only if tests show this does not satisfy the invariant.
- **R14 — Separate quality projects:** Analyze the Rust API and Vue frontend in separate SonarQube projects, including each service's tests and owned deployment configuration, and inspect both analyses through the configured SonarQube MCP.

## Current local state

On 2026-09-23, the owner deleted the `url-shortener` namespace to reduce host resource use. Read-only inspection confirmed that the namespace is absent and no PVCs or PVs remain in the cluster. The application is not deployed; its previous mappings are unavailable from the cluster. No external backup or recovery source has been verified. A fresh Helm install creates new Cassandra volumes and does not restore the old mappings.

## Scope boundaries

Included: anonymous local create-and-redirect, canonical deduplication, fixed hash-derived codes, immutable mappings, same-origin Vue UI, two frontend and two API replicas, horizontally partitioned Cassandra storage, Helm deployment, service-oriented source directories and guides, and a local kind deployment.

Excluded: accounts, private links, custom aliases, update/delete, expiration, analytics, public internet exposure, rate limiting, abuse reporting/takedown, production backup/recovery commitments, availability across independent failure domains, cache infrastructure, and production traffic/latency targets. Do not expose anonymous creation beyond local development until product and security owners define abuse controls.

## Acceptance checks

- Unit tests cover HTTP(S) parsing/canonicalization, byte-length boundaries, rejected schemes and credentials, deterministic Base62 output, and prefix extension.
- Integration tests use Cassandra to verify canonical deduplication, a forced code collision, simultaneous duplicate submissions, `201`/`200`/`301`/`404`/`503` behavior, cache headers, and resolving a mapping from a fresh API router after the original router is dropped.
- Helm lint/rendering verifies service and database resources, replica defaults, stable service DNS names, probes, local image tags, and Cassandra PVC configuration.
- Frontend unit tests cover success, validation/service errors, and copy feedback.
- Playwright E2E verifies keyboard-operable creation and copy, the 301 Location and cache policy, and an unknown-code 404 against the deployed application.
- The deployed frontend has two Ready Pods and the API has two Ready Pods, with distinct images and Deployments. Port-forwarding the frontend Service supports asset loading, API creation, Short Code redirect/404 behavior, and separate readiness checks; Cassandra runs as an independently scaled three-replica StatefulSet with three Bound PVCs.
- Concurrent create requests sent through both API replicas return one mapping for the same canonical Destination URL; collision extension remains unique across replicas. Redis is absent unless this invariant fails in verified tests.
- SonarQube analyses use separate `url-shortener-api` and `url-shortener-frontend` projects, each with the correct service paths, active findings reviewed, and quality gate results reported.
- Rust format/Clippy/tests, frontend typecheck/lint/tests/build, SonarQube full-project analysis, Helm lint/rendering, and any local kind flow are run and reported with actual outcomes.

## Remaining owner decisions

1. What authentication, abuse handling, reporting/takedown, and rate limits are required before public exposure?
2. What production Cassandra datacenter/rack layout, replication factor, consistency levels, backup/recovery targets, hostname, TLS, and availability goals should replace the local kind demo?
3. Is a one-hour redirect cache (`Cache-Control: public, max-age=3600`) acceptable for the immutable local MVP?

The first two questions do not block local-only implementation. The implementation uses a one-hour cache as the concrete cacheable `301` policy and documents it for review.
