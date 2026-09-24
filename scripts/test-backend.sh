#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
container_name="url-shortener-test-cassandra-$$"
coverage_mode=false

if [[ "${1:-}" == "--coverage" ]]; then
  coverage_mode=true
  shift
fi
if (($# > 0)); then
  printf 'Usage: %s [--coverage]\n' "${BASH_SOURCE[0]}" >&2
  exit 2
fi

for command in docker cargo python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Required command not found: %s\n' "$command" >&2
    exit 2
  fi
done

cleanup() {
  local exit_status=$?
  if (( exit_status != 0 )); then
    docker inspect --format \
      'Temporary Cassandra state={{.State.Status}} exit={{.State.ExitCode}} oom={{.State.OOMKilled}}' \
      "$container_name" >&2 2>/dev/null || true
    docker exec "$container_name" cqlsh 127.0.0.1 "$native_port" \
      -e 'SELECT broadcast_address, rpc_address, listen_address, native_protocol_version FROM system.local' \
      >&2 2>/dev/null || true
    docker logs --tail 80 "$container_name" >&2 2>/dev/null || true
  fi
  docker rm --force "$container_name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

native_port="$(python3 - <<'PY'
import socket

with socket.socket() as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
)"

docker run --detach --rm \
  --name "$container_name" \
  --publish "127.0.0.1:${native_port}:${native_port}" \
  --env CASSANDRA_CLUSTER_NAME=url-shortener-tests \
  --env CASSANDRA_BROADCAST_RPC_ADDRESS=127.0.0.1 \
  --env CASSANDRA_DC=datacenter1 \
  --env CASSANDRA_RACK=rack1 \
  --env CASSANDRA_ENDPOINT_SNITCH=GossipingPropertyFileSnitch \
  --env CASSANDRA_TEST_NATIVE_PORT="$native_port" \
  --env MAX_HEAP_SIZE=1G \
  --env HEAP_NEWSIZE=256M \
  --entrypoint bash \
  cassandra:5.0.9 \
  -ec 'sed -ri "s/^native_transport_port:.*/native_transport_port: ${CASSANDRA_TEST_NATIVE_PORT}/" "$CASSANDRA_CONF/cassandra.yaml"; exec /usr/local/bin/docker-entrypoint.sh cassandra -f' >/dev/null

for attempt in $(seq 1 180); do
  if docker exec "$container_name" cqlsh 127.0.0.1 "$native_port" \
    -e 'SELECT release_version FROM system.local' >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 180 ]]; then
    printf '%s\n' 'Temporary Cassandra 5.0.9 did not become ready.' >&2
    docker logs --tail 80 "$container_name" >&2 || true
    exit 1
  fi
  sleep 2
done

export CASSANDRA_CONTACT_POINTS="127.0.0.1:${native_port}"
export CASSANDRA_KEYSPACE=url_shortener_test
export CASSANDRA_LOCAL_DC=datacenter1
export CASSANDRA_REPLICATION_FACTOR=1

if [[ "$coverage_mode" == true ]]; then
  if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    printf '%s\n' 'Install cargo-llvm-cov 0.9.1 to produce the Rust coverage report.' >&2
    exit 2
  fi
  cargo llvm-cov \
    --manifest-path "$repository_root/url-shortener-api/Cargo.toml" \
    --all-targets \
    --locked \
    --lcov \
    --output-path "$repository_root/url-shortener-api/target/rust-coverage.lcov"
  python3 - "$repository_root/url-shortener-api/target/rust-coverage.lcov" "$repository_root" <<'PY'
from pathlib import Path
import sys

report_path = Path(sys.argv[1])
repository_root = Path(sys.argv[2]).resolve()
rewritten = []

for line in report_path.read_text(encoding="utf-8").splitlines(keepends=True):
    if not line.startswith("SF:"):
        rewritten.append(line)
        continue

    source_path = Path(line[3:].rstrip("\r\n"))
    if source_path.is_absolute():
        try:
            relative_path = source_path.resolve().relative_to(repository_root)
        except ValueError as error:
            raise SystemExit(f"Coverage source is outside the repository: {source_path}") from error
        rewritten.append(f"SF:{relative_path.as_posix()}\n")
    else:
        rewritten.append(line)

report_path.write_text("".join(rewritten), encoding="utf-8")
PY
else
  cargo test \
    --manifest-path "$repository_root/url-shortener-api/Cargo.toml" \
    --all-targets \
    --locked
fi
