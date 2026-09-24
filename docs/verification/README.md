# Verification index

This directory records browser-quality evidence for the URL Shortener. The existing design evidence remains the source for deployment and WebMCP-specific checks.

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Evidence

- [Lighthouse and SEO META verification](lighthouse.md) — production-build browser audit and manual metadata checklist.
- [SonarQube verification](sonarqube.md) — current re-analysis status and required project-scoped credentials.
- [WebMCP evidence](../design/webmcp/evidence.md) — mocked tool integration checks and native-browser verification limits.
- [Deployment evidence](../design/helm-deployment/evidence.md) — Helm and service verification.

Lighthouse browser scores, SonarQube static-analysis results, and JEV readiness judgments measure different things; report them as separate evidence.

## Document lifecycle

This index records verification links and has no independent build, deployment, or undeployment lifecycle.
