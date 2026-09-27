# ADR-0001: Use Rust for Backend Services

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner's request; historical team approval is not recorded.

> This record captures the Rust API already present in the repository. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

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

The URL Shortener has a Rust/Axum API that validates, creates, and resolves short links. It uses Cassandra for durable mappings and a Vue frontend for the user-facing flow. The backend handles concurrent HTTP requests and conditional writes that coordinate code allocation.

The project owner's rationale is that Rust is safer, fast, and can use less CPU and memory. The owner also recognizes Rust's deeper learning curve and considers it feasible with AI-assisted code authoring. These are design expectations and the owner's experience, not results from a comparative benchmark in this repository.

## Decision drivers

- Use compile-time ownership and type checks to reduce memory-safety and concurrency defects.
- Keep runtime overhead appropriate for a small API deployment; actual resource use must be measured.
- Implement concurrent request handling and Cassandra operations with explicit error behavior.
- Make Rust development feasible for this proof of concept through AI assistance, compiler feedback, focused tests, and human review.

## Options considered

### Rust for the API

- **Benefits:** Compiled native service, strong ownership and type checks, and a small runtime footprint are a good fit for a concurrent API.
- **Costs and risks:** Ownership, lifetimes, async types, and compiler diagnostics create a steeper learning curve; builds may take longer than a small scripting service.

### Go for the API

- **Benefits:** Static typing, approachable concurrency primitives, and a straightforward deployment model.
- **Costs and risks:** A migration would introduce churn without evidence that Rust is a project bottleneck; its garbage-collected runtime has different performance and safety trade-offs.

### TypeScript/Node.js for the API

- **Benefits:** Could reuse the frontend language and reduce language switching for contributors.
- **Costs and risks:** The service would use a managed runtime and would not receive Rust's ownership checks; CPU and memory behavior would need measurement rather than assumption.

No cross-language performance benchmark is available, so the comparison is qualitative.

## Decision outcome

Use Rust for the URL Shortener API. AI-assisted authoring makes the learning curve acceptable for this proof of concept when paired with compiler checks, automated tests, API-contract review, and human review. AI-generated code is not considered correct solely because it compiles or was produced by a capable model.

## Consequences

### Positive

- Rust's ownership and type system catch many memory-safety and concurrency mistakes before deployment.
- Native binaries and low runtime overhead are expected to fit the local service budget; representative profiling is still needed to verify actual consumption.
- AI assistance can help contributors navigate Rust while compiler diagnostics, tests, and review remain the verification controls.

### Negative and risks

- Contributors need time to learn Rust ownership, async APIs, and the codebase's domain rules.
- Local builds and CI require a Rust toolchain and locked dependencies.
- Reviewers must check URL validation, collision handling, persistence semantics, and error responses; generated code can violate domain invariants.

## Evidence and realization

- The [API manifest](../../url-shortener-api/Cargo.toml) identifies the Rust/Axum service.
- The [URL Shortener System Design](../design/url-shortener/design.md) records API behavior, Cassandra ownership, and code-allocation semantics.
- The [Kubernetes resource budget](../kubernetes-resources.md) documents configured CPU and memory requests and limits; it does not compare languages or report production consumption.

## Review triggers

- Reconsider if representative profiling shows the Rust API misses a measured performance or resource target and a controlled alternative-language comparison indicates a better fit.
- Reconsider if Rust's learning or maintenance cost remains a material delivery risk despite AI assistance, compiler tooling, tests, and review.
- Create a new ADR before migrating the backend to another language or runtime.

## References

- [URL Shortener README](../../README.md)
- [URL Shortener System Design](../design/url-shortener/design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
