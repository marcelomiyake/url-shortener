# `url-shortener` Helm chart

This chart installs the local URL shortener as one Helm release while keeping the frontend, API, and Cassandra in separate Kubernetes workloads:

> Documentation: [project index](../../../docs/README.md) · [repository overview](../../../README.md)

- two frontend replicas behind ClusterIP Service `url-shortener`
- two Rust API replicas behind internal ClusterIP Service `url-shortener-api`
- three Cassandra replicas with one 2 Gi persistent volume claim per pod

The chart is validated with Helm `4.3.0` and uses chart API `v2`. The `values.yaml` file holds image tags, replica counts, service names, resource requests/limits, Cassandra heap and storage settings. Keep image tags explicit and aligned with the images loaded into the local `kind` cluster.

Install Helm `4.3.0` from the [official releases](https://github.com/helm/helm/releases) and confirm `helm version --short` reports `v4.3.0` before using the project deployment script.

Install or upgrade from the repository root with:

```sh
helm upgrade --install url-shortener deploy/helm/url-shortener \
  --namespace url-shortener --create-namespace --wait --timeout 15m
```

For local builds, use [`scripts/deploy-kind.sh`](../../../scripts/deploy-kind.sh), which checks the existing cluster target, builds and loads the tagged images, and invokes Helm. It does not create, switch, upgrade, or delete the kind cluster.

Removing the Helm release retains the Cassandra PVCs through the StatefulSet retention policy. Deleting the namespace removes its PVC claims and can make mappings unavailable; this chart is not a backup system. There is no legacy database manifest or importer in this release.

## Build and use prerequisites

- **Build/deploy:** use the root project README for source-image build commands and the chart instructions below for the Helm release. Required tools are Docker, the supported local Kubernetes cluster/context, kubectl, and Helm.
- **Use:** the cluster release must be ready; use the loopback browser/port-forward instructions in the root README. See the resource table for each pod container budget.


## Resource budgets and undeploy

The API, frontend, and each Cassandra pod have CPU and memory requests and limits. See the per-pod values in the [Kubernetes resource budget](../../../docs/kubernetes-resources.md) and the chart's `values.yaml`. The Cassandra StatefulSet explicitly retains PVCs when deleted or scaled down. Remove the release with `helm uninstall url-shortener --namespace url-shortener`; deleting the namespace or retained PVCs removes mappings. See the [documentation index](../../../docs/README.md).


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
