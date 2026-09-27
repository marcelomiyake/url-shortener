# ADR-0002: Use Vue for the URL Shortener Frontend

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

The frontend provides the create form, result state, short-link navigation, and experimental WebMCP integration. NGINX serves the static bundle and proxies same-origin requests to the API. No original frontend framework selection record was found.

The scope of this decision is Vue 3 and TypeScript for the browser link-creation and redirect experience. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep link creation and result interactions in a small browser application.
- Use the same client flow for regular form use and the experimental browser-agent tool.
- Keep API validation and link allocation owned by the Rust backend.

## Options considered

### Vue 3 with TypeScript

- **Benefits:** The existing components and tests implement the form, result, and browser integration.
- **Costs and risks:** Vue-specific frontend tooling must be maintained alongside the Rust API toolchain.

### React with TypeScript

- **Benefits:** Could implement the same component and WebMCP flows.
- **Costs and risks:** Would require a rewrite without a documented limitation in the current UI.

### Server-rendered HTML

- **Benefits:** Could simplify a basic create form.
- **Costs and risks:** Would require redesigning the current reactive state and experimental browser tool integration.

## Decision outcome

Retain Vue 3 and TypeScript for the browser frontend. The API remains authoritative for validation, code allocation, and storage.

## Consequences

### Positive

- The frontend is a stateless API consumer and does not allocate codes independently.
- The browser form and WebMCP action share the project’s create flow.

### Negative and risks

- The WebMCP behavior is experimental and should not be confused with the core link API.
- Framework selection has no comparative benchmark or original decision record.

## Evidence and realization

- [package.json](../../url-shortener-frontend/package.json)
- [Dockerfile](../../url-shortener-frontend/Dockerfile)
- [README.md](../../url-shortener-frontend/README.md)
- [design.md](../design/url-shortener/design.md)

## Review triggers

- Reconsider if the browser platform or accessibility requirements cannot be met by the current application.
- Create a new ADR before replacing Vue or moving allocation logic into the client.

## References

- [url-shortener README](../../README.md)
- [System Design](../design/url-shortener/design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
