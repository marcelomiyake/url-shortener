# SonarQube verification

## Current re-analysis status

The URL Shortener scan was not rerun on 2026-09-25 because the configured scanner requires separate project-scoped `SONAR_API_TOKEN` and `SONAR_FRONTEND_TOKEN` variables, and neither is available in the current environment. The SonarQube server is reachable, but the existing general `SONAR_TOKEN` does not replace those repository-required credentials.

| Project | Last recorded coverage | Last recorded gate | Latest analysis date |
|---|---:|---|---|
| [`url-shortener-api`](http://127.0.0.1:9000/dashboard?id=url-shortener-api) | 90.2% | OK | 2026-09-23 |
| [`url-shortener-frontend`](http://127.0.0.1:9000/dashboard?id=url-shortener-frontend) | 90.6% | OK | 2026-09-23 |

These are the prior server results documented in the [URL Shortener evidence record](../design/url-shortener/evidence.md); they are not a fresh analysis of the current working tree. The current modifications are primarily documentation and frontend page metadata.

## Run the current analysis

Provide the project-scoped API and frontend analysis tokens to the shell, then run:

```sh
./scripts/sonar-scan.sh
```

The script runs the backend and frontend coverage checks and submits both SonarQube analyses. Do not commit the tokens or place them in command arguments.
