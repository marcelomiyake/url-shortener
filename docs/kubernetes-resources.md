# Kubernetes container resource budgets

These are chart-configured CPU, memory, and ephemeral-storage per-container local defaults, not measured consumption or production sizing claims. Cassandra's settings apply independently to each of its three pods.


> Project documentation index: [Documentation index](README.md)


## Workload budgets

| Pod/workload and container | CPU request | Memory request | Ephemeral request | CPU limit | Memory limit | Ephemeral limit | Source |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `url-shortener-api` / `api` | 100m | 128Mi | 128Mi | 1 | 512Mi | 512Mi | [Chart values](../deploy/helm/url-shortener/values.yaml) |
| `url-shortener` / `frontend` | 50m | 64Mi | 64Mi | 250m | 256Mi | 256Mi | [Chart values](../deploy/helm/url-shortener/values.yaml) |
| `cassandra` / `cassandra` | 500m | 1536Mi | 256Mi | 2 | 3Gi | 1Gi | [Chart values](../deploy/helm/url-shortener/values.yaml) |

Every chart workload container has CPU, memory, and ephemeral-storage requests and limits. Each Cassandra pod separately requests a 2Gi PVC; PVC storage is separate from its container budgets. No init containers or sidecars are defined.

## Kubernetes resource management practice

Set CPU, memory, and ephemeral-storage `requests` and `limits` on every container in every Pod, including init containers and sidecars. Requests guide scheduling and reserve baseline capacity; CPU limits may throttle, memory limits can trigger OOM termination, and ephemeral-storage limits constrain writable layers and temporary files. Measure representative usage, leave startup/burst headroom, monitor throttling and restarts, and right-size deliberately. Treat the listed values as local development defaults, not production sizing guidance. PVC storage requests are separate from container budgets.

## Build, use, and persistence

The [root README](../README.md) documents build, deploy, use, and undeploy commands with their persistent-data effects. The [Helm chart guide](../deploy/helm/url-shortener/README.md) is the deployment owner. This resource inventory is configuration guidance and does not itself deploy workloads.
