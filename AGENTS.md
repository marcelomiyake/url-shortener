# Agent guidance

> Human guide: [README.md](README.md) · [Documentation index](docs/README.md)

## Contents

- [Project intent](#project-intent)
- [Stack baseline](#stack-baseline)
- [Evidence and design workflow](#evidence-and-design-workflow)
- [Domain-driven design for agent work](#domain-driven-design-for-agent-work)
- [Agent job quality gate (DDD + Jev)](#agent-job-quality-gate-ddd-jev)
- [URL shortener decisions](#url-shortener-decisions)
- [Commit messages](#commit-messages)
- [Implementation conventions](#implementation-conventions)
- [Kubernetes and local `kind`](#kubernetes-and-local-kind)
- [SonarQube quality and duplication gate](#sonarqube-quality-and-duplication-gate)
- [Validation and handoff](#validation-and-handoff)

## Project intent

Maintain the URL-shortening project as a Rust API and Vue frontend deployable to the existing local `kind` cluster. The current repository contains both services, Cassandra schema, a Helm chart, tests, and deployment scripts; inspect the checkout before describing runtime state or assuming deployment succeeded.

Current top-level organization:

```text
url-shortener-api/       Rust API, API Dockerfile, tests
url-shortener-frontend/  Vue application, frontend Dockerfile, tests
database/cassandra/      Cassandra schema and database-owned scripts/configuration
deploy/helm/url-shortener/ Helm chart for the API, frontend, and Cassandra
docs/design/<feature>/   Design brief, evidence, proposal, and review artifacts
```

## Stack baseline

Versions below were checked against official release sources on 2026-09-23. Before creating or updating toolchain pins, recheck the official sources and use the newest supported stable/LTS patch available then. Do not use nightly, alpha, beta, release-candidate, or floating `latest` versions unless the user asks.

- **Rust:** stable 1.98.1. For a new workspace, pin the stable toolchain in `rust-toolchain.toml`, include `rustfmt` and `clippy`, and commit `Cargo.lock` for application crates.
- **Frontend runtime:** Node.js 24.21.0 LTS. Node 26 is Current, not LTS, as of the check date.
- **Vue:** Vue 3.5.43 stable. Use TypeScript for new Vue application code. Pin TypeScript 5.9.3 while the configured SonarQube Community Build analyzer supports TypeScript only through 5.9.3; recheck analyzer support before upgrading. Prefer the official Vue/Vite toolchain; do not add a state-management or UI library without a demonstrated need.
- **Cassandra:** Apache Cassandra 5.0.9 is pinned for local deployment and integration tests. The Rust service uses the Cassandra-compatible `scylla` CQL driver 1.9.0. Recheck both official release sources before changing pins.
- **Kubernetes:** 1.37.0 is the latest stable release as of the check date. The existing `kind` cluster may run another version: inspect it and target its actual server version when testing. Keep `kubectl` within the Kubernetes version-skew policy.
- **Helm:** stable `4.3.0` was checked on 2026-09-23 and supports Kubernetes `v1.37.x`; use the Helm `v2` chart API. Recheck the [official releases](https://github.com/helm/helm/releases) and [version-skew policy](https://docs.helm.sh/docs/topics/version_skew/) before updating the pin.
- **Package management:** use npm for a new frontend unless the repository already has a package-manager convention. Commit `package-lock.json` and use `npm ci` for reproducible installs.

Sources: [Rust releases](https://blog.rust-lang.org/releases/), [Node.js releases](https://nodejs.org/en/about/previous-releases), [Vue changelog](https://github.com/vuejs/core/blob/main/CHANGELOG.md), [Kubernetes releases](https://kubernetes.io/releases/), and [Kubernetes version-skew policy](https://kubernetes.io/releases/version-skew-policy/).

For Cassandra, see [Apache Cassandra downloads](https://cassandra.apache.org/_/download) and the [Scylla Rust driver releases](https://github.com/scylladb/scylla-rust-driver/releases).

## Evidence and design workflow

- Before changing behavior or proposing architecture, inspect the relevant source, manifests, contracts, migrations, tests, and operational documentation. Record the repository revision when preparing a design review.
- Separate **confirmed facts**, **proposals**, **assumptions**, and **open questions**. Cite repository paths and symbols/sections for claims about current behavior. Preserve conflicting or stale evidence until it is resolved; do not fill gaps with plausible-sounding components or provider guarantees.
- For nontrivial design work, create a short feature brief first under `docs/design/<feature>/brief.md`. Give requirements stable IDs (`R1`, `R2`, …); include goal, proposed targets (separate from measured results), existing constraints, out-of-scope items, and open questions. Do not silently decide unresolved product or security policy.
- Gather evidence before drafting a design. Compare viable alternatives, explain trade-offs and capacity arithmetic with units and assumptions, identify what needs a prototype or measurement, and record proposed decisions and their reconsideration conditions. Add API contracts, diagrams, ADRs, and review checklists when they help answer the design question; keep them consistent with the same decisions and requirement IDs.
- Reuse verified existing infrastructure where it meets the requirements. Do not introduce a database, cache, queue, extra service, or deployment dependency based only on convention or a component name.
- Keep design approval distinct from structural checks. Linting or rendering does not prove security, isolation, recovery, performance, or product correctness. Mark checks as executed only with actual results; leave a proposal marked as a proposal until the responsible reviewers approve it.
- Keep `AGENTS.md` focused on durable repository instructions. Put feature-specific requirements in the feature brief and human-facing architecture/setup explanations in the README or design documents.

## Domain-driven design for agent work

- Use DDD to clarify the business problem and ownership; do not add DDD patterns as ceremony. Establish the actor, desired outcome, and a precise ubiquitous language with repository evidence and the relevant product/domain owner. Keep terms consistent within a bounded context; allow a term to mean something different in another context when its model differs.
- Identify bounded contexts from model, language, consistency, and ownership boundaries. Do not equate a bounded context with a folder, team, service, or agent job. Keep coherent behavior together; split work only when the boundary and independent outcome are clear. For cross-context work, name the owner on each side, authoritative data, contract, dependency order, and failure behavior.
- For tasks that change domain behavior, state the command/use case, business invariants, authoritative data owner, state transitions, and required consistency. Use aggregates or value objects only when they protect identified domain rules; keep simpler flows simple.
- Mark disputed terms, rules, ownership, and context boundaries as open questions. Assign each decision to a product/domain owner. An agent or a score must not turn an unresolved business policy into an accepted requirement.

## Agent job quality gate (DDD + Jev)

Apply this gate to every non-trivial agent job before execution or delegation. A job is an explicit, bounded request for work, not just a broad goal. Include:

- The actor, business outcome, bounded context, and domain terms relevant to the work.
- Repository evidence and source revision; distinguish confirmed behavior from proposals and assumptions.
- In-scope and out-of-scope changes, dependencies, affected owners, and any cross-context contract.
- Deliverables and observable acceptance criteria, including the commands, fixtures, or human decisions that can verify them.
- Relevant invariants, failure cases, data/safety limits, required permissions or approvals, and a clear stop condition.
- Open questions, with an owner or discovery action for each. Unknowns are acceptable when the job is explicitly to investigate or resolve them; they are not permission to guess.

When an authorized TypeSafe Jev client/API is available, score the job with one independent `Score` question per dimension below, using the same minimal state containing the job brief, cited evidence, and context/domain notes. Ask the independent questions together. Use the ordered descriptions as the Score criteria without numeric labels. Jev is a structured evaluator; it does not replace the coding agent, write the job, or prove that a high-scoring job is correct.

| Score dimension | Lowest level | Level 1 | Level 2 | Ready level |
| --- | --- | --- | --- | --- |
| Domain fit | The requested outcome conflicts with confirmed behavior or is unsafe for the domain. | Actor, outcome, or applicable context is missing; technical labels obscure the business need. | Actor and context are named, but important terms or rules are ambiguous or unlabeled. | Actor, outcome, and context are explicit; terms are consistent and unresolved domain choices are visible. |
| Scope and evidence | Work is contradictory, unbounded, or depends on unsupported claims. | Scope, sources, or dependencies needed to start are missing. | The main scope is clear, but in/out boundaries, source evidence, ownership, or a cross-context contract is incomplete. | Scope, exclusions, evidence, owners, dependencies, and relevant contracts are explicit; assumptions are separated from facts. |
| Deliverable and acceptance | There is no observable outcome to hand off. | The job describes activity (such as “implement it”) without a verifiable result. | A deliverable is named, but one or more acceptance checks are subjective or unavailable. | Deliverables and observable acceptance checks are traceable to requirements and can be run or reviewed. |
| Risk and decision ownership | The job exceeds authorization, hides a material risk, or requires an unsafe action. | Affected data, failure modes, or decision owners are omitted. | Main risks are identified, but permission boundaries, recovery, owner, or stop/approval condition is unclear. | Data and permission limits, failure handling, decision owners, and stop/approval conditions are explicit. |

Treat the table's columns as ordered Score criteria (levels 0–3). The initial gate is **score ≥ 2.5 and probability on levels 0–2 ≤ 0.20 for every required dimension**. Do not average dimensions to hide a weak critical area. Review the full probability distribution: a low-confidence or split result suggests overlap or missing context, not proof that the job is wrong or right. Calibrate this initial threshold against human-reviewed jobs before using it for automatic dispatch decisions.

If any dimension fails, improve the job itself by adding evidence, narrowing scope, defining acceptance, or assigning the unresolved decision to its owner; do not make up domain facts to raise a score. Re-score after each focused revision, with at most two revisions. If the gate still fails, do not dispatch implementation work that depends on the gap; ask the user/domain owner a specific question or return the job as blocked with the missing evidence and decision stated.

If TypeSafe access is not configured or authorized, run the same rubric as a manual checklist and state that Jev was not called. Never fabricate Jev scores or confidence, add a runtime TypeSafe dependency to this URL-shortener solely for agent review, expose an API key to the Vue app, or send credentials, customer data, or unrelated private source to the evaluator. Recheck the live [Score documentation](https://docs.typesafe.ai/primitives/score), [Jev for coding agents](https://docs.typesafe.ai/introduction/coding-agents), and [composite scoring guidance](https://docs.typesafe.ai/patterns/composite-scoring) before implementing a TypeSafe integration.

After a job completes, compare its changes and actual verification evidence to the same requirements and acceptance criteria. A good job score assesses the request's quality; it does not certify the implementation or replace design, security, or product review.

## URL shortener decisions

Approved local-MVP policy is recorded in `docs/design/url-shortener/brief.md`: immutable URL mappings, deterministic SHA-256/Base62 codes, canonical URL deduplication, Cassandra as the authoritative store, and cacheable 301 redirects. Keep changes aligned with that brief. Do not infer public exposure, production availability, backup/recovery, rate limiting, analytics, or abuse policy from those local decisions; capture new requirements and owner decisions before extending scope.

Treat submitted destination URLs as untrusted input. Do not fetch destinations from the backend as part of shortening or redirecting. Define and validate the accepted URL schemes explicitly; never execute or render a submitted URL as active HTML/JavaScript. Avoid logging full destination URLs unless the need and data-handling policy are established.

Keep the backend as the source of truth for link creation and redirect behavior. Define the API contract before coupling frontend behavior to endpoint paths, response shapes, redirect status, or error semantics. If OpenAPI is introduced, keep it aligned with the implementation and consumer-facing examples.

For browser-agent support, use WebMCP only as progressive frontend enhancement. Reuse the existing Short Link use case and keep visible UI state synchronized. Treat registration as optional, unregister with component/page lifecycle, minimize returned data, and require an approved design before exposing tools across origins. Persistent link creation is consequential; use the current WebMCP annotation when supported without treating it as a guarantee of user confirmation. Keep browser support and manual verification evidence explicit because the API is evolving.

## Commit messages

- Follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) for every commit. Use `<type>[optional scope]: <description>`; add `!` after the type or scope for a breaking change, and use the `BREAKING CHANGE: ...` footer when describing its impact. Add a body or other footers when useful.

## Implementation conventions

- Preserve the service-oriented layout. The Rust API lives in `url-shortener-api/`; the Vue application lives in `url-shortener-frontend/`; each has its own Dockerfile, README, and AGENTS guide. Keep both independently buildable; avoid additional runtime services without a documented reason.
- Keep Cassandra schema and database-owned scripts/configuration in `database/cassandra/`, outside the API service. Keep Helm deployment settings under the chart's `cassandra:` values and API connection environment handling in the API service. The API compiles the table definition with `include_str!`; build its container with repository-root Docker context as documented in the root README and deployment script.
- Use stable Rust and safe, idiomatic error handling in request paths. Run formatting, Clippy, and Cargo checks using the workspace's actual manifests and documented commands.
- Keep the frontend accessible and responsive. Use the project's actual npm scripts for type checking, linting, tests, and production builds; do not claim a command exists if `package.json` does not define it.
- Use [OpenDesign](https://github.com/nexu-io/open-design) for frontend design work: maintain `url-shortener-frontend/DESIGN.md` and review a browser prototype before substantial Vue UI changes. Treat OpenDesign as design-time tooling, not a Vue component library or runtime dependency. Translate accepted design into Vue/TypeScript and verify accessibility and responsive behavior.
- Keep each service independently buildable, with a service-local `README.md` and `AGENTS.md`. Keep the project-wide domain, design, Kubernetes, Sonar, and validation rules in this root guide. Put service implementation conventions and commands in the service-local guides.
- Keep dependency versions reproducible with committed lockfiles. Update toolchain and dependency pins deliberately, and report compatibility changes.
- Do not invent ports, routes, environment variables, health endpoints, database schemas, secrets, or external service addresses. Confirm each from code/configuration or record it as a proposal.
- Never commit credentials, tokens, private URLs, or real customer data. Keep secrets out of frontend bundles and source control.

## Kubernetes and local `kind`

- Before any cluster operation, inspect `kubectl config current-context`, `kind get clusters`, `kubectl cluster-info`, and the server/node versions. Use only the existing local `kind` context for requested local deployment checks. If the active context is not clearly the intended `kind` cluster, stop before applying changes and report the mismatch.
- Do not create, delete, upgrade, or switch clusters/contexts as part of routine project work. Do not target a remote cluster. Keep test resources in a dedicated project namespace and remove only resources created for that test.
- Use the chart in `deploy/helm/url-shortener/` for application and database workloads. Use `scripts/deploy-kind.sh` as the local install/upgrade path; it requires Helm `4.3.0` and runs `helm upgrade --install`. Do not keep parallel Kustomize deployment definitions for the same workloads.
- Use currently served Kubernetes API versions (`apps/v1`, `v1`, and `networking.k8s.io/v1` when applicable). Do not assume an Ingress controller or other add-on is installed; inspect the cluster first.
- Use explicit, non-`latest` image tags that match the image loaded into `kind`. When local images are needed, build or pull them and load them into the verified cluster with `kind load docker-image`; do not push local test images to a registry.
- Configure readiness/liveness/startup probes only for implemented endpoints. Set resource requests and limits, run containers as non-root where supported, and pass secrets through Kubernetes Secrets or the project's established secret mechanism rather than baking them into images or manifests.
- For a requested deployment check, report the cluster context and version, commands actually run, observed rollout/pod results, and any checks not performed. A successful `kubectl apply` alone is not proof that the application works.
- The local topology targets two frontend Deployment replicas, two API Deployment replicas, and three Cassandra StatefulSet replicas with one PVC per Cassandra pod. Cassandra uses `NetworkTopologyStrategy`, local replication factor 3, `LOCAL_QUORUM` ordinary reads/writes, and `LOCAL_SERIAL` conditional writes. Redis is not part of the allocation design: deterministic candidates plus Cassandra `IF NOT EXISTS` protect each Short Code partition across API replicas.
- The active chart contains no legacy relational database or import job. The prior cutover was completed before the namespace was deleted; the current cluster has no project PVC/PV and prior mappings are unavailable. Do not claim recovery or create import tooling without an identified backup/source and an owner-approved brief. Helm's PVC-retention policy protects claims on StatefulSet removal, but namespace deletion removes namespaced claims; preserve backups outside the namespace.

## SonarQube quality and duplication gate

- For every task that changes source code or project configuration, use the configured SonarQube MCP to inspect `marcelomiyake_url-shortener` before remediation and again after changes. Confirm the project, branch, and analyzed revision; do not assume a previous or partial analysis represents the current checkout.
- SonarQube Cloud project `marcelomiyake_url-shortener` is connected to GitHub with Automatic Analysis enabled. Push fixes to the default branch or a pull request, wait for analysis, confirm the analyzed revision, and inspect the full project issue list and quality gate through the Cloud UI or MCP. No local scanner script, config file, or token is required. Automatic Analysis does not import local coverage reports.
- Review all active findings across the project, including findings that predate the current task or lie outside the changed files. Remediate actionable legacy issues as well as newly introduced ones. Continue the inspect-fix-analyze loop until the current full-project analysis has no outstanding actionable findings and the configured quality gate passes.
- Review bugs, vulnerabilities, security hotspots, code smells, duplicated code, and other configured gate conditions. Read each finding's rule, location, severity, and context before changing code. Fix the underlying cause; do not silence rules, weaken quality profiles, add exclusions, or dismiss findings just to obtain a passing gate. Treat a finding as a false positive only when the code and rule evidence support that conclusion and the project's review process permits it; record the rationale.
- Avoid duplicated code in new changes. When SonarQube reports duplication, check whether the code represents the same rule and language in the same bounded context. Extract a focused shared function/type when behavior and ownership are truly shared; keep similar-looking behavior separate when its domain meaning or invariants differ. Do not introduce a generic abstraction solely to reduce a duplication metric.
- Preserve behavior while cleaning up. Add or update focused checks when an issue fix changes behavior, run the repository's documented validation, and inspect the final diff. If a safe fix requires an unresolved product/domain decision, explain the finding and ask its owner rather than hiding it.
- Do not report the project as clean based on a quality-gate badge from an older revision, a scan limited to changed files, or passing unit tests alone. Record the analyzed project/branch/revision, remaining findings, quality-gate result, and checks actually run in the handoff.
- If SonarQube MCP is unavailable, misconfigured, or cannot analyze the current revision, state that the required quality verification is incomplete. Do not claim a clean scan or substitute a different tool as if it were SonarQube; request the MCP access/configuration needed to complete the gate.

## Validation and handoff

- Use commands documented in the repository and relevant manifests. If documentation is missing, inspect the available scripts and tool configuration before selecting checks; add/update the canonical commands as part of scaffolding rather than leaving guessed commands here.
- Do not add or run tests unless the user asks for testing/verification or the requested implementation requires it. When checks are requested, report exact commands, outcomes, and limits; distinguish structural validation, automated tests, local cluster checks, and human design review.
- At handoff, summarize changed files, evidence and decisions, actual validation results, and unresolved assumptions with the next decision or owner needed. Do not claim approval, readiness, or deployment without evidence.
