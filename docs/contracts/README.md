# API and data contract catalog

This catalog records the Short Link HTTP and Cassandra boundaries. The [OpenAPI definition](../design/url-shortener/openapi.yaml), CQL schema, and service implementation are authoritative.

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Contents

- [Contracts](#contracts)
- [Consumer map](#consumer-map)
- [Compatibility and security](#compatibility-and-security)
- [Related documentation](#related-documentation)
- [Document lifecycle](#document-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Contracts

| Contract | Type and authority | Owner | Producer | Consumers |
| --- | --- | --- | --- | --- |
| Short Link HTTP API (`POST /api/v1/links`, `GET /{code}`) | REST/JSON and redirect; [OpenAPI](../design/url-shortener/openapi.yaml) | `url-shortener` / `url-shortener-api` | `url-shortener` / `url-shortener-api` | `url-shortener` / `url-shortener-frontend`; other-repository consumers are unknown; none is recorded in the tracked repositories. |
| WebMCP `create-short-link` tool | Browser tool contract; [`url-shortener-frontend/src/services/webmcp.ts`](../../url-shortener-frontend/src/services/webmcp.ts) and the Short Link HTTP API | `url-shortener` / `url-shortener-frontend` | `url-shortener` / `url-shortener-frontend` registers the tool in the active document | A browser agent invoking the same-page `document.modelContext`; specific agent products are unknown. No cross-origin exposure is configured. |
| Short Link mapping schema | Cassandra CQL; [schema](../../database/cassandra/schema/short_links_by_code.cql) | `url-shortener` / Cassandra database assets, with schema application owned by `url-shortener-api` | `url-shortener` / `url-shortener-api` writes mappings | `url-shortener` / `url-shortener-api` reads/writes; frontend does not access Cassandra. |
| Frontend same-origin paths | HTTP proxy behavior; [NGINX configuration](../../url-shortener-frontend/nginx.conf) | `url-shortener` / `url-shortener-frontend` owns routing only | `url-shortener` / Browser | `url-shortener` / `url-shortener-api` receives proxied `/api/*` and `/{code}` requests. |

## Consumer map

The frontend is the known API client. The WebMCP `create-short-link` browser tool reuses the frontend's same-origin create flow; it is not another server API. Other-repository consumers are unknown; none is recorded in the tracked repositories.

## Compatibility and security

The API returns `201` for new mappings, `200` for canonical duplicates, `422` for destination policy failures, and `503` for storage failures. Mappings are immutable, redirects are cacheable, and there is no revocation flow. The service never fetches the submitted destination and does not log it.

## Related documentation

- [System Design](../design/url-shortener/design.md)
- [Project README](../../README.md)
- [Documentation index](../README.md)

## Document lifecycle

This contract catalog is maintained as Markdown and links to the implementation-owned interface definitions. It has no independent software build, deployment, or undeployment lifecycle. Review it when its linked contracts or consumers change.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
