# WebMCP Frontend Tools — Feature Brief

**Status:** Approved for implementation by the project owner's request (2026-09-23).
**Evidence baseline:** worktree based on `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`).
**Decision owner:** project owner.
**Bounded context:** Short Link. The browser UI owns the interaction state; the Rust API remains authoritative for validation, canonicalization, allocation, and persistence.

> Project documentation index: [Documentation index](../../README.md)

## Contents

- [Outcome and language](#outcome-and-language)
- [Confirmed evidence](#confirmed-evidence)
- [Requirements](#requirements)
- [Scope boundaries](#scope-boundaries)
- [Acceptance checks](#acceptance-checks)
- [Source evidence](#source-evidence)

## Outcome and language

Allow a browser agent to help the page's user create a **Short Link** by invoking one well-defined in-page tool. A Short Link is a public, immutable mapping from a short code to an HTTP(S) Destination URL. The WebMCP tool must reuse the frontend's existing create request and keep the visible Vue UI in sync with its result.

## Confirmed evidence

- `url-shortener-frontend/src/App.vue` owns the destination form, loading/error/result state, and same-origin short-link URL construction.
- `url-shortener-frontend/src/services/links.ts:createShortLink` is the frontend's existing same-origin call to `POST /api/v1/links` and validates the response before returning it.
- The API creates or reuses an immutable, public Short Link; no delete operation exists. Creation is a persistent state change.
- WebMCP's current imperative API is `document.modelContext.registerTool()`. The API is optional and remains an evolving web platform proposal. Chrome documents a local-development flag and an origin trial; unsupported browsers must keep the regular form usable. Official type definitions are published separately as `webmcp-types`.
- The feature uses only the current page origin. It will not register `exposedTo` origins or send destinations to an external agent service.

## Requirements

- **R1 — Shared use case:** Register a `create-short-link` tool with one string input, `destinationUrl`. Reuse the same API client and UI state as human form submission. Return only the resulting same-origin Short URL and update the visible result panel.
- **R2 — Clear action and risk:** Describe that the action creates or reuses a public, non-expiring link. Mark it `consequentialHint: true`; do not claim this annotation guarantees a browser confirmation prompt.
- **R3 — Least exposure:** Register only in the active page's `document.modelContext`, use no cross-origin `exposedTo`, and unregister on component teardown. Do not register tools that reveal destinations, enumerate stored links, or delete data.
- **R4 — Progressive enhancement:** If `document.modelContext` is absent or registration is rejected, leave the Vue form operational. Use `webmcp-types` only as a development/type dependency; do not add a runtime SDK or polyfill.
- **R5 — Reliable state and errors:** Validate tool input at runtime, reuse API validation, reflect loading/success/failure in the user-visible state, honor WebMCP cancellation where possible, and return a concise actionable error without exposing server internals or echoing submitted destinations.
- **R6 — Verification and documentation:** Add focused unit coverage for tool registration, invocation, cancellation/lifecycle, and unsupported browsers. Document setup, browser support limits, security decisions, and manual verification in the frontend guide and feature evidence.

## Scope boundaries

Included: the frontend tool registration and cleanup, reuse of existing link creation behavior, TypeScript definitions, focused tests, and frontend/root documentation. Excluded: backend changes, a server-side MCP service, authentication/authorization changes, link lookup/history/deletion tools, cross-origin tool exposure, origin-trial enrollment, and live-browser verification without a WebMCP-capable browser session.

## Acceptance checks

- `npm run typecheck`, `npm run lint`, `npm run test:unit`, `npm run test:unit:coverage`, and `npm run build` pass.
- Unit tests show that a tool invocation calls the existing API client once, updates visible app state, returns the Short URL, and is unregistered on teardown; unsupported/rejected WebMCP registration does not break human form submission.
- Manual browser verification instructions identify Chrome's current WebMCP flag/origin-trial requirements and use the official Model Context Tool Inspector. Do not claim browser-agent support was exercised unless it was actually tested.
- The WebMCP action does not log or return the submitted Destination URL and is not exposed to additional origins.

## Source evidence

- [WebMCP explainer and imperative API](https://github.com/webmachinelearning/webmcp)
- [Chrome WebMCP setup and browser support](https://developer.chrome.com/docs/ai/webmcp)
- [WebMCP best practices](https://developer.chrome.com/docs/ai/webmcp/best-practices)
- [Security-minded WebMCP tools](https://developer.chrome.com/docs/ai/webmcp/secure-tools)
- [Official WebMCP TypeScript definitions](https://github.com/webmachinelearning/webmcp-types)
