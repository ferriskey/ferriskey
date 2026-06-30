# ferriskey-operator

![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 0.7.0](https://img.shields.io/badge/AppVersion-0.7.0-informational?style=flat-square)

A Helm chart for the Ferriskey Kubernetes Operator

## Overview

The Ferriskey Operator manages `FerrisKeyCluster` instances declaratively through a Kubernetes Custom Resource Definition (CRD). This chart deploys the operator itself — the CRD, the operator Deployment, and the required RBAC resources.

To deploy a Ferriskey cluster once the operator is running, create a `FerrisKeyCluster` resource:

```yaml
apiVersion: ferriskey.rs/v1alpha1
kind: FerrisKeyCluster
metadata:
  name: my-ferriskey
  namespace: ferriskey
spec:
  name: my-ferriskey
  replicas: 1
  version: "0.7.0"
  api:
    apiUrl: "https://api.iam.example.com"
    webappUrl: "https://iam.example.com"
    allowedOrigins:
      - "https://iam.example.com"
  database:
    secretRef:
      name: ferriskey-db-credentials
    databaseName: ferriskey
    sslMode: require
```

The database secret must contain the fields `host`, `port`, `user`, and `password`.

## Installation

```sh
helm install ferriskey-operator oci://ghcr.io/ferriskey/charts/ferriskey-operator \
  --namespace ferriskey-system \
  --create-namespace
```

## CRD Management

By default (`crds.install: true`), the CRD is installed as part of this chart and annotated with `helm.sh/resource-policy: keep` (`crds.keep: true`) to prevent accidental deletion on `helm uninstall`.

If you manage CRDs externally (e.g. via GitOps or a separate CRD chart), set:

```yaml
crds:
  install: false
```

## RBAC

By default (`rbac.create: true`), the chart creates a `ClusterRole` and `ClusterRoleBinding` granting the operator the permissions it needs to manage `FerrisKeyCluster` resources across all namespaces.

If you manage RBAC externally, set:

```yaml
rbac:
  create: false
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| common.affinity | object | `{}` | Common affinity for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| common.annotations | object | `{}` | Common annotations for all workloads. |
| common.dnsConfig | object | `{}` | Common DNS config for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| common.env | list | `[]` | Common environment variables for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| common.envFrom | list | `[]` | Common environment variables from sources for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| common.hostAliases | list | `[]` | Common host aliases for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| common.image.pullPolicy | string | `"IfNotPresent"` | Default pull policy for all images. |
| common.image.tag | string | `nil` | Default tag for all images. Defaults to the chart's appVersion. |
| common.imagePullSecrets | list | `[]` | Common image pull secrets for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| common.initContainers | list | `[]` | Common init containers for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| common.labels | object | `{}` | Common labels for all workloads. |
| common.podAnnotations | object | `{}` | Common annotations for all pods. |
| common.podLabels | object | `{}` | Common labels for all pods. |
| common.revisionHistoryLimit | int | `nil` | Default revision history limit for all workloads. |
| common.runtimeClassName | string | `nil` | Default runtime class name for all pods. |
| common.schedulerName | string | `nil` | Default scheduler name for all pods. |
| common.serviceAccount.annotations | object | `{}` | Common annotations on all service accounts. |
| common.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token by default for all service accounts. |
| common.serviceAccount.labels | object | `{}` | Common labels on all service accounts. |
| common.tolerations | list | `[]` | Common tolerations for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| common.topologySpreadConstraints | list | `[]` | Common topology spread constraints for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| common.volumeMounts | list | `[]` | Common volume mounts for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| common.volumes | list | `[]` | Common volumes for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| crds.install | bool | `true` | Install the FerrisKeyCluster CRD as part of this chart. Set to `false` if you manage CRDs externally (e.g. via GitOps or a separate CRD chart). |
| crds.keep | bool | `true` | Keep the CRD on chart uninstall by annotating it with `helm.sh/resource-policy: keep`. Strongly recommended in production to avoid accidental data loss. |
| nameOverride | string | `nil` | Override the name of the release. |
| operator.affinity | object | `{}` | Affinity for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| operator.annotations | object | `{}` | Annotations on the operator workload. |
| operator.args | list | `[]` | Arguments for the operator container. |
| operator.command | list | `[]` | Command for the operator container. |
| operator.dnsConfig | object | `{}` | DNS config for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| operator.env | list | `[]` | Environment variables for the operator container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| operator.envFrom | list | `[]` | Environment variables from sources for the operator container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| operator.hostAliases | list | `[]` | Host aliases for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| operator.hostIPC | bool | `nil` | Use host's IPC namespace for the operator pods. |
| operator.hostNetwork | bool | `nil` | Use host's network namespace for the operator pods. |
| operator.hostPID | bool | `nil` | Use host's PID namespace for the operator pods. |
| operator.hostUsers | bool | `nil` | Use host's user namespace for the operator pods. |
| operator.image.pullPolicy | string | `nil` | Pull policy for the operator image. |
| operator.image.repository | string | `"ghcr.io/ferriskey/ferriskey-operator"` | Repository for the operator image. |
| operator.image.tag | string | `nil` | Tag for the operator image. Defaults to the chart's appVersion. |
| operator.imagePullSecrets | list | `[]` | Image pull secrets for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| operator.initContainers | list | `[]` | Init containers for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| operator.labels | object | `{}` | Labels on the operator workload. |
| operator.livenessProbe | object | `{"exec":{"command":["/bin/sh","-c","kill -0 1"]},"failureThreshold":3,"initialDelaySeconds":10,"periodSeconds":30,"timeoutSeconds":5}` | Liveness probe for the operator container. Uses a process-level check since the operator does not expose an HTTP health endpoint. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| operator.nodeName | string | `nil` | Node name for the operator pods. |
| operator.nodeSelector | object | `{}` | Node selector for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#nodeselector-v1-core |
| operator.podAnnotations | object | `{}` | Annotations on the operator pods. |
| operator.podLabels | object | `{}` | Labels on the operator pods. |
| operator.podSecurityContext | object | `{"fsGroup":1000,"runAsGroup":1000,"runAsNonRoot":true,"runAsUser":1000,"seccompProfile":{"type":"RuntimeDefault"}}` | Security context for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#podsecuritycontext-v1-core |
| operator.preemptionPolicy | string | `nil` | Preemption policy for the operator pods. |
| operator.priority | int | `nil` | Priority for the operator pods. |
| operator.priorityClassName | string | `nil` | Priority class name for the operator pods. |
| operator.replicas | int | `1` | Number of replicas for the operator. Operators should run as a single replica. Scale via leader election, not replicas. |
| operator.resources | object | `{"limits":{"memory":"256Mi"},"requests":{"memory":"64Mi"}}` | Resources for the operator container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#resourcerequirements-v1-core |
| operator.revisionHistoryLimit | int | `nil` | Revision history limit for the operator workload. |
| operator.runtimeClassName | string | `nil` | Runtime class name for the operator pods. |
| operator.schedulerName | string | `nil` | Scheduler name for the operator pods. |
| operator.securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true}` | Security context for the operator container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#securitycontext-v1-core |
| operator.serviceAccount.annotations | object | `{}` | Annotations on the operator service account. |
| operator.serviceAccount.automountServiceAccountToken | bool | `true` | Automount the service account token. Must be true — the operator needs the token to call the Kubernetes API. |
| operator.serviceAccount.create | bool | `true` | Create a service account for the operator. |
| operator.serviceAccount.labels | object | `{}` | Labels on the operator service account. |
| operator.serviceAccount.name | string | `nil` | Name of the service account. Defaults to the operator workload name. |
| operator.tolerations | list | `[]` | Tolerations for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| operator.topologySpreadConstraints | list | `[]` | Topology spread constraints for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| operator.volumeMounts | list | `[]` | Volume mounts for the operator container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| operator.volumes | list | `[]` | Volumes for the operator pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| rbac.create | bool | `true` | Create the ClusterRole and ClusterRoleBinding required by the operator. Set to `false` if you manage RBAC externally. |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.14.2](https://github.com/norwoodj/helm-docs/releases/v1.14.2)
