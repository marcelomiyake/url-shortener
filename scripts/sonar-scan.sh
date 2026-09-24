#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
scanner_image="${SONAR_SCANNER_IMAGE:-sonarsource/sonar-scanner-cli@sha256:a3f4215076706c95a17a68c19322ee916e40a3acd081a8c1a1e839e0194afa57}"
sonar_host_url="${SONAR_HOST_URL:-${SONAR_URL:-http://host.docker.internal:9000}}"

if [[ -z "${SONAR_API_TOKEN:-}" || -z "${SONAR_FRONTEND_TOKEN:-}" ]]; then
  printf '%s\n' \
    'Set SONAR_API_TOKEN and SONAR_FRONTEND_TOKEN to project-scoped analysis tokens before scanning.' >&2
  exit 2
fi
if ! command -v cargo-llvm-cov >/dev/null 2>&1 || [[ "$(cargo llvm-cov --version)" != 'cargo-llvm-cov 0.9.1' ]]; then
  printf '%s\n' 'Install the pinned coverage tool with: cargo install cargo-llvm-cov --version 0.9.1 --locked' >&2
  exit 2
fi

"$repo_root/scripts/test-backend.sh" --coverage
(
  cd "$repo_root/url-shortener-frontend"
  npm ci
  npm run test:unit:coverage
)

cargo clippy \
  --manifest-path "$repo_root/url-shortener-api/Cargo.toml" \
  --all-targets \
  --locked \
  --message-format=json \
  > "$repo_root/url-shortener-api/target/clippy-report.json"

scan_project() {
  local properties_file="$1"
  local project_token="$2"
  shift 2
  SONAR_TOKEN="$project_token" docker run --rm \
    --add-host=host.docker.internal:host-gateway \
    --env SONAR_TOKEN \
    --env "SONAR_HOST_URL=$sonar_host_url" \
    --volume "$repo_root:/usr/src" \
    --workdir /usr/src \
    "$scanner_image" \
    -Dproject.settings="$properties_file" \
    -Dsonar.qualitygate.wait=true \
    "$@"
}

scan_project sonar-project-api.properties "$SONAR_API_TOKEN" "$@"
scan_project sonar-project-frontend.properties "$SONAR_FRONTEND_TOKEN" "$@"
