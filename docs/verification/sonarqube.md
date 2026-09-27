# SonarQube Cloud verification

The Rust `url-shortener-api` has its own SonarQube Cloud project. The [GitHub Actions workflow](https://github.com/marcelomiyake/url-shortener/actions/workflows/sonarcloud-microservices.yml) runs coverage and analysis automatically on pushes to `main`; `workflow_dispatch` supports a manual rerun. Organization-level auto-import of new GitHub repositories is disabled, so this workflow manages the service project.

The workflow provisions Cassandra, runs `cargo llvm-cov --lcov --output-path target/coverage/lcov.info -- --test-threads=1` from `url-shortener-api`, and imports the report with `cargo sonar-scanner`. It reads the repository secret named `SONAR_TOKEN`. The startup entrypoint `src/main.rs` is excluded from the coverage denominator. The Vue frontend is not part of this SonarQube project, and no local Sonar scan script is used.

## Project and coverage

Coverage below is SonarCloud's overall line coverage for `main`, not new-code or local coverage. The baseline is the latest Cloud result before the coverage-test updates; the current column is the latest result after them. Values are from the 2026-09-27 snapshot.

| Microservice | SonarCloud project | Before | Current | Change |
| --- | --- | ---: | ---: | ---: |
| `url-shortener-api` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_url-shortener_url-shortener-api) | 96.2% | 98.6% | +2.4 pp |

The project has a passing Quality Gate, zero open or confirmed issues, zero hotspots awaiting review, zero bugs, zero vulnerabilities, zero code smells, and 0.0% duplicated lines.

## Verification

Confirm the latest workflow completed for the pushed commit and that the project `main` analysis matches that revision. Review coverage, active issues, security hotspots, duplication, and the Quality Gate. The project link above opens the live Cloud dashboard; the workflow link shows its run history. Local checks do not replace a completed Cloud analysis.
