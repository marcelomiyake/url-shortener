#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
kind_context="kind-kind"
kind_cluster="kind"
namespace="url-shortener"
release="url-shortener"
chart="$repo_root/deploy/helm/url-shortener"
api_image="url-shortener-api:0.2.0"
frontend_image="url-shortener-frontend:0.1.0"
cassandra_image="cassandra:5.0.9"

for command in docker kind kubectl helm; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Required command not found: %s\n' "$command" >&2
    exit 2
  fi
done

helm_version="$(helm version --short | cut -d+ -f1)"
if [[ "$helm_version" != "v4.3.0" ]]; then
  printf 'Helm %s is required; found %s.\n' 'v4.3.0' "$helm_version" >&2
  exit 2
fi

current_context="$(kubectl config current-context)"
if [[ "$current_context" != "$kind_context" ]]; then
  printf 'Refusing deployment: current context is %s; expected the existing local context %s.\n' \
    "$current_context" "$kind_context" >&2
  exit 2
fi
if ! kind get clusters | grep -Fxq "$kind_cluster"; then
  printf 'Refusing deployment: existing kind cluster %s was not found.\n' "$kind_cluster" >&2
  exit 2
fi

printf 'Verified Kubernetes target before changes:\n'
kubectl --context "$kind_context" cluster-info
kubectl --context "$kind_context" version
kubectl --context "$kind_context" get nodes -o wide
kubectl --context "$kind_context" get storageclass standard >/dev/null

node_architecture="$(kubectl --context "$kind_context" \
  get nodes -o jsonpath='{.items[0].status.nodeInfo.architecture}')"
case "$node_architecture" in
  amd64|arm64) image_platform="linux/$node_architecture" ;;
  *)
    printf 'Unsupported kind node architecture for image export: %s\n' \
      "$node_architecture" >&2
    exit 2
    ;;
esac

printf '\nBuilding API image %s with repository-root context...\n' "$api_image"
docker build --platform "$image_platform" --tag "$api_image" \
  --file "$repo_root/url-shortener-api/Dockerfile" "$repo_root"
printf 'Building frontend image %s...\n' "$frontend_image"
docker build --platform "$image_platform" --tag "$frontend_image" \
  --file "$repo_root/url-shortener-frontend/Dockerfile" \
  "$repo_root/url-shortener-frontend"
printf 'Pulling pinned Cassandra image %s...\n' "$cassandra_image"
docker pull --platform "$image_platform" "$cassandra_image"

image_archive_dir="$(mktemp -d)"
trap 'rm -rf -- "$image_archive_dir"' EXIT
for image in "$api_image" "$frontend_image" "$cassandra_image"; do
  image_archive="$image_archive_dir/${image%%:*}.tar"
  docker image save --platform "$image_platform" --output "$image_archive" "$image"
  kind load image-archive "$image_archive" --name "$kind_cluster"
done
rm -rf -- "$image_archive_dir"
trap - EXIT

printf '\nInstalling/upgrading Helm release %s in namespace %s...\n' \
  "$release" "$namespace"
helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --create-namespace \
  --wait \
  --timeout 15m

printf '\nHelm release status:\n'
helm status "$release" --namespace "$namespace"
printf '\nDeployed resources:\n'
kubectl --context "$kind_context" --namespace "$namespace" get pods,services,pvc
printf '\nReplica counts:\n'
kubectl --context "$kind_context" --namespace "$namespace" \
  get deployment url-shortener-api url-shortener-frontend -o wide
kubectl --context "$kind_context" --namespace "$namespace" \
  get statefulset cassandra -o wide
cat <<'SUMMARY'

Frontend URL: http://127.0.0.1:8080
Start the local port-forward in another terminal:
  kubectl --context kind-kind --namespace url-shortener port-forward service/url-shortener 8080:8080
Run browser end-to-end tests from url-shortener-frontend with:
  npm run test:e2e
SUMMARY
