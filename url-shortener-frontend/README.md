# `url-shortener-frontend`

Accessible Vue 3 and TypeScript client for creating anonymous Short Links. It serves as static assets from an unprivileged NGINX image. The browser talks to the Rust API through same-origin `/api/*` and `/{code}` proxy paths; the frontend does not own link validation or persistence.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

## Contents

- [Run and verify](#run-and-verify)
- [Design and deployment](#design-and-deployment)
- [Browser agent support (WebMCP)](#browser-agent-support-webmcp)
- [Component ownership, prerequisites, and lifecycle](#component-ownership-prerequisites-and-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Run and verify

From this directory:

```sh
npm ci
npm run dev
```

The project uses Vite for local development and proxies `/api` to `http://127.0.0.1:8080`. Run its checks with:

```sh
npm run typecheck
npm run lint
npm run test:unit
npm run test:unit:coverage
npm run build
```

Playwright end-to-end tests target the deployed local service. Start a port-forward to the frontend Service, then run `npx playwright install chromium` once and `BASE_URL=http://127.0.0.1:8080 npm run test:e2e`.

## Design and deployment

[`DESIGN.md`](DESIGN.md) records the project-owned Neutral Modern design handoff based on OpenDesign. OpenDesign is design-time tooling and is not included in the runtime image. `Dockerfile` builds a static production bundle and packages it with NGINX; [`nginx.conf`](nginx.conf) defines same-origin proxying and process health routes. The Helm chart runs two frontend replicas behind the `url-shortener` ClusterIP Service.

See [`AGENTS.md`](AGENTS.md) for frontend-specific contribution rules and the root [`AGENTS.md`](../AGENTS.md) for project-wide design, Kubernetes, and quality gates.

## Browser agent support (WebMCP)

When the browser supports WebMCP, the page registers one imperative `document.modelContext` tool named `create-short-link`. It accepts an HTTP(S) `destinationUrl`, calls the same create flow as the Vue form, updates the visible result panel, and returns only the resulting same-origin Short URL. The tool is marked `consequentialHint: true` because link creation persists a public, non-expiring mapping. This is a hint to agents and browsers; it does not guarantee a confirmation prompt. The tool is not shared with additional origins, and the submitted destination is not returned in the tool result.

WebMCP is experimental and browser-dependent. The current Chrome documentation describes a Chrome 149 origin trial and a local testing flag. For local testing, run the frontend and API, open `chrome://flags/#enable-webmcp-testing`, enable the flag, relaunch Chrome, then load the local frontend. Use Chrome's [Model Context Tool Inspector](https://developer.chrome.com/docs/ai/webmcp) to verify the registered tool, invoke it with a test destination, and confirm the UI and returned Short URL agree. The regular form remains available when WebMCP is missing or registration is denied. WebMCP support is not claimed for browsers that have not implemented the API.

The browser integration uses the current `document.modelContext` API and the pinned `webmcp-types` `0.1.9` development-only TypeScript package. It does not add an agent SDK, polyfill, server-side MCP, or runtime dependency. See the [feature brief](../docs/design/webmcp/brief.md) for requirements and [evidence](../docs/design/webmcp/evidence.md) for implementation and verification status. Recheck the official [WebMCP explainer](https://github.com/webmachinelearning/webmcp), [implementation status](https://github.com/webmachinelearning/webmcp/blob/main/implementation-status.md), [best practices](https://developer.chrome.com/docs/ai/webmcp/best-practices), and [tool security guidance](https://developer.chrome.com/docs/ai/webmcp/secure-tools) before changing this integration because the API is evolving.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `url-shortener` / `url-shortener-frontend`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/design/url-shortener/design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/url-shortener/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
