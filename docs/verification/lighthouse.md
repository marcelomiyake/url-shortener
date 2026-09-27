# Lighthouse and SEO META Verification

This record audits the built URL Shortener frontend and checks its browser metadata against the published SEO META in 1 CLICK field list. Lighthouse measures this browser run; it is separate from SonarQube analysis and JEV readiness evaluation.

## Contents

- [Scope](#scope)
- [Lighthouse results](#lighthouse-results)
- [Agentic Browsing](#agentic-browsing)
- [SEO META checklist](#seo-meta-checklist)
- [Findings](#findings)
- [Related documentation](#related-documentation)

## Scope

- **Date/time:** 2026-09-25 12:57:42 America/Sao_Paulo (final metadata recheck).
- **Source:** dirty worktree based on revision 90f5478.
- **Environment:** Linux; Google Chrome 154.0.8037.57; Lighthouse CLI 13.5.0; default mobile emulation and simulated throttling.
- **Target:** production Vite preview at http://127.0.0.1:4173/; one app preview ran at a time and was stopped before the next.
- **Commands:** `npm run build`; `npx --yes lighthouse http://127.0.0.1:4173/ --output=json --output-path=/tmp/lighthouse-url-shortener-final.json --only-categories=performance,accessibility,best-practices,seo --chrome-flags='--headless --no-sandbox' --quiet`.
- The API was not deployed; the audit covers the initial frontend page, not link creation against the API.

## Lighthouse results

| Performance | Accessibility | Best practices | SEO |
| ---: | ---: | ---: | ---: |
| 100 | 100 | 100 | 63 |

Scores range from 0 to 100. The SEO score reflects the intentional noindex directive.

## Agentic Browsing

Lighthouse 13.5.0 on Chrome 154.0.8037.57 ran `npx --yes lighthouse http://127.0.0.1:4173/ --output=json --output-path=/tmp/lighthouse-url-shortener-agentic.json --only-categories=agentic-browsing --chrome-flags='--headless --no-sandbox' --quiet` at 2026-09-25 12:51:10 America/Sao_Paulo. Its experimental fractional result was 0.50; this is not a 0–100 score. The accessibility-tree and layout-stability audits passed. The root `llms.txt` and `ai-catalog.json` discovery checks failed because no public agent catalog is shipped for this local demo. WebMCP form coverage, registered tools, and schema validity were reported as not applicable: this run did not use the WebMCP origin trial, so it did not verify native registration of `create-short-link`.

## SEO META checklist

| Field | Result |
| --- | --- |
| HTML language | English is declared. |
| Title and length | Present: “Short Form — Create a short link” (32 characters). |
| Description and length | Present: “Create and share a short link.” (30 characters). |
| Robots metadata | noindex, nofollow is intentional for this local demo. A valid robots.txt is served with Allow: / and preserves Disallow rules for `/api/` and `/health/`; Lighthouse's robots.txt audit passed. |
| Canonical URL | Omitted because the local preview/deployment origin is not a stable public URL. |
| Headings | One H1 followed by the form/result H2 sections. |
| Images and alt text | No HTML image elements; this check is not applicable. |
| Links | No anchors appear before link creation; the result state adds the generated short-link anchor. |
| Favicon | SVG favicon is declared and served. |
| Open Graph, Twitter, and sitemap | Not provided because the app has no public share URL or public indexing target. |

The SEO META in 1 CLICK extension was not installed in the browser. This is a manual checklist audit against its published fields, not a claim that the extension itself ran.

## Findings

- Lighthouse confirmed the title, description, valid robots.txt, favicon, and page metadata. It reports the page as not crawlable by design because of the noindex directive.
- All four Lighthouse categories scored 100 except SEO, which scored 63 for the deliberate noindex policy.
- The existing WebMCP create tool remains consequential and experimental; its mocked integration tests do not establish native browser-agent interoperability.

## Related documentation

- [Documentation index](../README.md)
- [URL Shortener frontend guide](../../url-shortener-frontend/README.md)
- [WebMCP implementation evidence](../design/webmcp/evidence.md)
- [System and quality evidence](../design/url-shortener/evidence.md) — historical SonarQube results and JEV review limitations.
- [Helm deployment evidence](../design/helm-deployment/evidence.md)
- [Chrome Lighthouse overview](https://developer.chrome.com/docs/lighthouse/overview)
- [Lighthouse Agentic Browsing scoring](https://developer.chrome.com/docs/lighthouse/agentic-browsing/scoring)
- [SEO META in 1 CLICK listing](https://chromewebstore.google.com/detail/seo-meta-in-1-click/bjogjfinolnhfhkbipphpdlldadpnmhc?hl=en-GB)
