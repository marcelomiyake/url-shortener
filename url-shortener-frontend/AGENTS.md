# Frontend service guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

Read the root [`AGENTS.md`](../AGENTS.md) before making project changes. This service owns the browser experience and static asset delivery. Rust remains authoritative for destination policy, Short Code allocation, persistence, and redirect behavior.

## Frontend changes

- Use Vue 3 and TypeScript with the established Vite/npm toolchain. Keep the UI accessible, responsive, and aligned with [`DESIGN.md`](DESIGN.md).
- Treat the API response as data. Validate relative Short Paths before combining them with the current origin; render destinations and service messages as text, never as active HTML.
- Keep browser calls same-origin through `/api/v1/links`. Do not embed database addresses, tokens, or backend secrets in the bundle.
- Keep user-visible loading, validation, network, copy, and API failure states understandable. Avoid adding a UI or state library without a demonstrated need.
- Treat WebMCP as progressive enhancement: feature-detect `document.modelContext`, register a small number of focused tools, and unregister them when their page/component lifecycle ends. Keep the Vue flow usable when WebMCP is unsupported or registration is rejected.
- Review the WebMCP tool as a public API surface. Reuse existing frontend use cases, validate runtime inputs, synchronize visible state, return only the minimum output, honor cancellation where possible, and do not expose tools to other origins without a new approved design. Mark persistent or consequential actions with supported annotations; do not claim that metadata guarantees confirmation.
- WebMCP is an evolving browser proposal. Use the current imperative `document.modelContext` API and pinned `webmcp-types` for TypeScript. Do not add a polyfill/runtime agent SDK. Check the official spec, implementation status, best practices, and security guidance before changing the integration.
- Keep OpenDesign in the design workflow only. Do not copy unreviewed upstream templates or assets into runtime source.

## Checks

From this directory run `npm ci`, `npm run typecheck`, `npm run lint`, `npm run test:unit`, `npm run test:unit:coverage`, and `npm run build`. Run `BASE_URL=http://127.0.0.1:8080 npm run test:e2e` while port-forwarding the deployed frontend Service. The root Sonar script analyzes frontend separately from the Rust API. Report actual checks and outcomes. Use the WebMCP unit tests for registration/lifecycle behavior; claim browser-agent verification only after testing in a supported browser with WebMCP enabled.
