# WebMCP Frontend Tools — Evidence

**Worktree base revision:** `e241fef92eb7b8e71d4efc0ba40d52fec8ad187c` (`main`).
**Evidence date:** 2026-09-23.
**Requirements:** [brief.md](brief.md).
**State:** Worktree changes are uncommitted. Browser-agent interoperability has not been manually exercised.

> Project documentation index: [Documentation index](../../README.md)

## Implementation

- Added pinned development-only `webmcp-types` `0.1.9` and loaded its declarations in the frontend app TypeScript configuration. It adds no runtime SDK or polyfill.
- `src/services/webmcp.ts` feature-detects the optional `document.modelContext` argument by accepting `undefined`; it registers only `create-short-link`, with one required `destinationUrl` string. No cross-origin `exposedTo` option is provided.
- The tool calls the same App request handler as the human form. Success updates the destination field and visible result panel before returning the same-origin Short URL. The response omits the submitted destination.
- The tool carries `consequentialHint: true` because creation writes a public, non-expiring mapping. This annotation communicates risk but does not guarantee that an agent or browser will prompt for confirmation.
- The component unregisters its tool on unmount. Registration failure is caught and unregisters the failed registration; the ordinary Vue form remains available. WebMCP invocation cancellation is forwarded to `fetch`; cancellation messages explain that a request may have completed and that repeating it reuses the same mapping.

## Checks run

In `url-shortener-frontend/`:

- `npm ci`, `npm run typecheck` — passed.
- `npm run lint` — passed.
- `npm run test:unit` — passed: 3 test files, 13 tests. Tests cover tool registration/annotations, invocation arguments and response, origin-minimal output, missing input, denied registration fallback, component UI synchronization and lifecycle cleanup, plus cancellation forwarding.
- `npm run test:unit:coverage` — passed: 89.41% line coverage, 90.54% branch coverage, 94.11% function coverage (frontend unit-test scope).
- `npm run build` — passed.
- `npm install --save-dev --save-exact webmcp-types@0.1.9` — completed; npm reported 0 vulnerabilities. A later `npm ci` reproduced the dependency tree successfully and also reported 0 vulnerabilities.

The project's shared SonarQube scan was attempted, but stopped before analysis because `SONAR_API_TOKEN` and `SONAR_FRONTEND_TOKEN` project-scoped credentials were unavailable. See the [Helm evidence record](../helm-deployment/evidence.md) for MCP snapshot limitations and the previous analysis revision.

## Browser support and manual test status

The [official Chrome WebMCP documentation](https://developer.chrome.com/docs/ai/webmcp) describes an origin trial in Chrome 149 and a local `chrome://flags/#enable-webmcp-testing` switch. The WebMCP proposal is evolving and support varies by browser. Tests here use a mocked `document.modelContext` and prove the app integration, not compatibility with a native browser agent. Manual verification should follow the steps in the [frontend README](../../../url-shortener-frontend/README.md#browser-agent-support-webmcp) in an enabled browser with the official Model Context Tool Inspector.

## Manual DDD/Jev review

- **Domain fit:** the browser user and agent collaborate in the Short Link context; Rust remains authoritative for mapping rules.
- **Scope and evidence:** one create/reuse action calls the established same-origin API; no history, destination disclosure, deletion, or cross-origin tool sharing was added.
- **Deliverable and acceptance:** TypeScript, lint, unit, coverage, and production-build checks passed; browser-agent testing remains explicitly unverified.
- **Risk and ownership:** public immutable writes are marked consequential, cancellations account for uncertain write completion, and the project owner owns expansion of permissions or browser-support scope.

No TypeSafe/Jev client was available, so Jev was not called and no score or probability is claimed.
