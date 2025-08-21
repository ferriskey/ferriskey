# ferriskey

![Version: 0.2.0](https://img.shields.io/badge/Version-0.2.0-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 0.2.0](https://img.shields.io/badge/AppVersion-0.2.0-informational?style=flat-square)

A Helm chart for Ferriskey

## Installation

### With embedded PostgreSQL

**This configuration should not be used in production.**

#### Helm

```sh
helm repo add ferriskey oci://ghcr.io/ferriskey/charts
helm install ferriskey ferriskey/ferriskey
```

#### ArgoCD

If you're using ArgoCD, you need to set the following values:

```yaml
database:
  passwordSecret:
    annotations:
      argocd.argoproj.io/hook: PreSync
      argocd.argoproj.io/sync-wave: "0"
databaseMigrations:
  annotations:
    argocd.argoproj.io/hook: PreSync
    argocd.argoproj.io/sync-wave: "1"
postgresql:
  annotations:
    argocd.argoproj.io/hook: PreSync
    argocd.argoproj.io/sync-wave: "0"
  serviceAccount:
    annotations:
      argocd.argoproj.io/hook: PreSync
      argocd.argoproj.io/sync-wave: "0"
```

### With external PostgreSQL

#### Helm

Replace `$DATABASE_HOST` and `$SECRET_NAME` with the values of your database host and secret name.

```sh
helm repo add ferriskey oci://ghcr.io/ferriskey/charts
helm install ferriskey ferriskey/ferriskey \
    --set database.host=$DATABASE_HOST \
    --set database.passwordSecret.create=false \
    --set database.passwordSecret.name=$SECRET_NAME \
    --set postgresql.enabled=false
```

#### ArgoCD

Replace `$DATABASE_HOST` and `$SECRET_NAME` with the values of your database host and secret name.

```yaml
database:
  host: $DATABASE_HOST
  passwordSecret:
    create: false
    name: $SECRET_NAME
databaseMigrations:
  annotations:
    argocd.argoproj.io/hook: PreSync
postgresql:
  enabled: false
```

By default, ArgoCD will generate the secret for the admin password. You need to configure ArgoCD to ignore the password difference.

Example:
```yaml
  ignoreDifferences:
    - group: ""
      kind: Secret
      name: ferriskey-dev-api-admin
      jsonPointers:
        - /data/password
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| api.admin.email | string | `"admin@cluster.local"` | Email for the admin user. |
| api.admin.passwordSecret.annotations | object | `{}` | Annotations on the secret for the admin password. Used only if `create` is `true`. |
| api.admin.passwordSecret.create | bool | `true` | Create a secret for the admin password. |
| api.admin.passwordSecret.key | string | `"password"` | Key in the secret to use for the admin password. |
| api.admin.passwordSecret.labels | object | `{}` | Labels on the secret for the admin password. Used only if `create` is `true`. |
| api.admin.passwordSecret.name | string | `nil` | Name of the secret to use for the admin password. |
| api.admin.username | string | `"admin"` | Username for the admin user. |
| api.affinity | object | `{}` | Affinity for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| api.annotations | object | `{}` | Annotations on API workloads. |
| api.args | list | `[]` | Arguments for the API container. |
| api.command | list | `[]` | Command for the API container. |
| api.dnsConfig | object | `{}` | DNS config for the API workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| api.emptyDirSize | string | `nil` | EmptyDir size for the API workload. |
| api.env | list | `[]` | Environment variables for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| api.envFrom | list | `[]` | Environment variables from sources for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| api.environment | string | `"production"` | [DEPERECATED] Environment to run the API in. |
| api.ephemeralContainers | list | `[]` | Ephemeral containers for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ephemeralcontainer-v1-core |
| api.hostAliases | list | `[]` | Host aliases for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| api.hostIPC | bool | `nil` | Use host's IPC namespace for the API pods. |
| api.hostNetwork | bool | `nil` | Use host's network namespace for the API pods. |
| api.hostPID | bool | `nil` | Use host's PID namespace for the API pods. |
| api.hostUsers | bool | `nil` | Use host's user namespace for the API pods. |
| api.hostname | string | `nil` | Hostname for the API pods. |
| api.image.pullPolicy | string | `nil` | Pull policy for the image. |
| api.image.repository | string | `"ghcr.io/ferriskey/ferriskey-api"` | Repository for the image to use. |
| api.image.tag | string | `nil` | Tag for the image to use. Default to the chart's app version. |
| api.imagePullSecrets | list | `[]` | Image pull secrets for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| api.initContainers | list | `[]` | Init containers for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| api.labels | object | `{}` | Labels on API workloads. |
| api.lifecycle | object | `{}` | Lifecycle for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#lifecycle-v1-core |
| api.livenessProbe | object | `{"failureThreshold":3,"httpGet":{"path":"/api/health/live","port":"http"},"initialDelaySeconds":30,"periodSeconds":10,"timeoutSeconds":5}` | Liveness probe for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| api.log.filter | string | `"info"` | Log filter to use for the API (https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives). |
| api.log.json | bool | `false` | Whether to log in JSON format. |
| api.nodeName | string | `nil` | Node name for the API pods. |
| api.nodeSelector | object | `{}` | Node selector for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#nodeselector-v1-core |
| api.podAnnotations | object | `{}` | Annotations on API pods. |
| api.podLabels | object | `{}` | Labels on API pods. |
| api.podSecurityContext | object | `{"fsGroup":1000,"runAsGroup":1000,"runAsNonRoot":true,"runAsUser":1000,"seccompProfile":{"type":"RuntimeDefault"}}` | Security context for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#podsecuritycontext-v1-core |
| api.preemptionPolicy | string | `nil` | Preemption policy for the API pods. |
| api.priority | int | `nil` | Priority for the API pods. |
| api.priorityClassName | string | `nil` | Priority class name for the API pods. |
| api.readinessProbe | object | `{"failureThreshold":3,"httpGet":{"path":"/api/health/ready","port":"http"},"initialDelaySeconds":5,"periodSeconds":5,"successThreshold":1,"timeoutSeconds":3}` | Readiness probe for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| api.replicas | int | `1` | Number of replicas for the API workload. |
| api.resources | object | `{"limits":{"memory":"512Mi"},"requests":{"memory":"128Mi"}}` | Resources for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#resourcerequirements-v1-core |
| api.revisionHistoryLimit | int | `nil` | Revision history limit for the API workload. |
| api.runtimeClassName | string | `nil` | Runtime class name for the API pods. |
| api.schedulerName | string | `nil` | Scheduler name for the API pods. |
| api.securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true}` | Security context for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#securitycontext-v1-core |
| api.server.allowedOrigins | list | `[]` | Allowed origins for the server. |
| api.server.port | int | `3333` | Port for the server. |
| api.server.rootPath | string | `"/api"` | Root path for the server. If you edit it, you need to update the liveness and readiness probes. |
| api.service.annotations | object | `{}` | Annotations on the service for the API pods. |
| api.service.clusterIP | string | `nil` | Cluster IP for the service for the API pods. |
| api.service.clusterIPs | list | `[]` | Cluster IPs for the service for the API pods. |
| api.service.externalIPs | list | `[]` | External IPs for the service for the API pods. |
| api.service.externalName | string | `nil` | External name for the service for the API pods. |
| api.service.externalTrafficPolicy | string | `nil` | ExternalTrafficPolicy for the service for the API pods. |
| api.service.healthCheckNodePort | int | `nil` | Health check node port for the service for the API pods. |
| api.service.internalTrafficPolicy | string | `nil` | InternalTrafficPolicy for the service for the API pods. |
| api.service.ipFamilies | list | `[]` | IP families for the service for the API pods. |
| api.service.ipFamilyPolicy | string | `nil` | IP family policy for the service for the API pods. |
| api.service.labels | object | `{}` | Labels on the service for the API pods. |
| api.service.loadBalancerClass | string | `nil` | Load balancer class for the service for the API pods. |
| api.service.loadBalancerIP | string | `nil` | Load balancer IP for the service for the API pods. |
| api.service.loadBalancerSourceRanges | list | `[]` | Load balancer source ranges for the service for the API pods. |
| api.service.publishNotReadyAddresses | bool | `nil` | Publish not ready addresses for the service for the API pods. |
| api.service.sessionAffinity | string | `nil` | Session affinity for the service for the API pods. |
| api.service.sessionAffinityConfig | object | `{}` | Session affinity config for the service for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#sessionaffinityconfig-v1-core |
| api.service.trafficDistribution | string | `nil` | Traffic distribution for the service for the API pods. |
| api.service.type | string | `nil` | Type for the service for the API pods. |
| api.serviceAccount.annotations | object | `{}` | Annotations on the service account for the API pods. |
| api.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token for the API service account. |
| api.serviceAccount.create | bool | `true` | Create a service account for the API pods. |
| api.serviceAccount.labels | object | `{}` | Labels on the service account for the API pods. |
| api.serviceAccount.name | string | `nil` | Name of the service account for the API pods. Default is the API workload name. |
| api.serviceMonitor.annotations | object | `{}` | Annotations on the service monitor for the API service. |
| api.serviceMonitor.enableHttp2 | bool | `nil` | Enable HTTP/2 for the service monitor for the API service. |
| api.serviceMonitor.enabled | bool | `false` | Enable the service monitor for the API service. |
| api.serviceMonitor.filterRunning | bool | `nil` | Filter running pods for the service monitor for the API service. |
| api.serviceMonitor.followRedirects | bool | `nil` | Follow redirects for the service monitor for the API service. |
| api.serviceMonitor.honorLabels | bool | `nil` | Honor labels for the service monitor for the API service. |
| api.serviceMonitor.honorTimestamps | bool | `nil` | Honor timestamps for the service monitor for the API service. |
| api.serviceMonitor.interval | string | `"30s"` | Interval for the service monitor for the API service. |
| api.serviceMonitor.labels | object | `{}` | Labels on the service monitor for the API service. |
| api.serviceMonitor.metricsRelabelings | list | `[]` | Metrics relabelings for the service monitor for the API service. https://prometheus-operator.dev/docs/api-reference/api/#monitoring.coreos.com/v1.RelabelConfig |
| api.serviceMonitor.noProxy | string | `nil` | Comma separated list of domains/IPs/CIDRs excluded from proxying. |
| api.serviceMonitor.proxyConnectHeaders | object | `{}` | Connect headers for the proxy of the service monitor for the API service. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.31/#secretkeyselector-v1-core |
| api.serviceMonitor.proxyFromEnvironment | bool | `nil` | Use HTTP_PROXY, HTTPS_PROXY and NO_PROXY environment variables for the service monitor for the API service. |
| api.serviceMonitor.proxyUrl | string | `nil` | Proxy URL for the service monitor for the API service. |
| api.serviceMonitor.relabelings | list | `[]` | Relabelings for the service monitor for the API service. https://prometheus-operator.dev/docs/api-reference/api/#monitoring.coreos.com/v1.RelabelConfig |
| api.serviceMonitor.scrapeTimeout | string | `"30s"` | Scrape timeout for the service monitor for the API service. |
| api.serviceMonitor.trackTimestampsStaleness | bool | `nil` | Track timestamps staleness for the service monitor for the API service. |
| api.setHostnameAsFQDN | bool | `nil` | Set hostname as FQDN for the API pods. |
| api.shareProcessNamespace | bool | `nil` | Share a single process namespace between all of the containers for the API pods. |
| api.subdomain | string | `nil` | Subdomain for the API pods. |
| api.tolerations | list | `[]` | Tolerations for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| api.topologySpreadConstraints | list | `[]` | Topology spread constraints for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| api.volumeMounts | list | `[]` | Volume mounts for the API container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| api.volumes | list | `[]` | Volumes for the API pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| api.webapp.protocol | string | `"https"` | Protocol for the webapp. Ignored if `api.webapp.url` is set. |
| api.webapp.url | string | `nil` | URL for the webapp. Default computed from the ingress configuration. |
| common.affinity | object | `{}` | Common affinity for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| common.annotations | object | `{}` | Common annotations for all workloads. |
| common.dnsConfig | object | `{}` | Common DNS config for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| common.emptyDirSize | string | `"100Mi"` | Default size for emptyDir volumes. |
| common.env | list | `[]` | Common environment variables for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| common.envFrom | list | `[]` | Common environment variables from sources for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| common.ephemeralContainers | list | `[]` | Common ephemeral containers for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ephemeralcontainer-v1-core |
| common.hostAliases | list | `[]` | Common host aliases for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| common.image.pullPolicy | string | `"IfNotPresent"` | Default pull policy for all images. |
| common.image.tag | string | `nil` | Default tag for all images. Default to the chart's app version. |
| common.imagePullSecrets | list | `[]` | Common image pull secrets for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| common.initContainers | list | `[]` | Common init containers for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| common.labels | object | `{}` | Common labels for all workloads. |
| common.persistentVolumeClaimRetentionPolicy | object | `{"whenDeleted":"Retain","whenScaled":"Retain"}` | Common persistent volume claim retention policy for all workloads. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#statefulsetpersistentvolumeclaimretentionpolicy-v1-apps |
| common.podAnnotations | object | `{}` | Common annotations for all pods. |
| common.podLabels | object | `{}` | Common labels for all pods. |
| common.revisionHistoryLimit | int | `nil` | Default revision history limit for all workloads. |
| common.runtimeClassName | string | `nil` | Default runtime class name for all pods. |
| common.schedulerName | string | `nil` | Default scheduler name for all pods. |
| common.service.annotations | object | `{}` | Common annotations on all services. |
| common.service.externalTrafficPolicy | string | `nil` | Default external traffic policy for all services. |
| common.service.internalTrafficPolicy | string | `nil` | Default internal traffic policy for all services. |
| common.service.labels | object | `{}` | Common labels on all services. |
| common.service.loadBalancerClass | string | `nil` | Default load balancer class for all services. |
| common.service.loadBalancerSourceRanges | list | `[]` | Common load balancer source ranges for all services. |
| common.service.publishNotReadyAddresses | bool | `nil` | Publish not ready addresses by default for all services. |
| common.service.sessionAffinity | string | `nil` | Default session affinity for all services. |
| common.service.sessionAffinityConfig | object | `{}` | Common session affinity config for all services. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#sessionaffinityconfig-v1-core |
| common.service.trafficDistribution | string | `nil` | Default traffic distribution for all services. |
| common.service.type | string | `"ClusterIP"` | Default type for all services. |
| common.serviceAccount.annotations | object | `{}` | Common annotations on the service account. |
| common.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token by default for all service accounts. |
| common.serviceAccount.labels | object | `{}` | Common labels on all service accounts. |
| common.setHostnameAsFQDN | bool | `nil` | Set by default hostname as FQDN for all pods. |
| common.shareProcessNamespace | bool | `nil` | Share by default a single process namespace between all of the containers in all pods. |
| common.subdomain | string | `nil` | Default subdomain for all pods. |
| common.tolerations | list | `[]` | Common tolerations for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| common.topologySpreadConstraints | list | `[]` | Common topology spread constraints for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| common.updateStrategy | object | `{"type":"RollingUpdate"}` | Common update strategy for all workloads. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#statefulsetupdatestrategy-v1-apps |
| common.volumeMounts | list | `[]` | Common volume mounts for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| common.volumes | list | `[]` | Common volumes for all pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| database.host | string | `nil` | Host for the database. |
| database.name | string | `"ferriskey"` | Name for the database. |
| database.passwordSecret.annotations | object | `{}` | Annotations on the secret for the database password. Used only if `database.passwordSecret.create` is `true`. |
| database.passwordSecret.create | bool | `true` | Generate a secret for database password. |
| database.passwordSecret.key | string | `"password"` | Key in the secret to use for the database password. |
| database.passwordSecret.labels | object | `{}` | Labels on the secret for the database password. Used only if `database.passwordSecret.create` is `true`. |
| database.passwordSecret.name | string | `nil` | Name of the secret to use for the database password. |
| database.port | int | `5432` | Port for the database. |
| database.user | string | `"ferriskey"` | User for the database. |
| databaseMigrations.affinity | object | `{}` | Affinity for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| databaseMigrations.annotations | object | `{}` | Annotations on the database migrations job. |
| databaseMigrations.args | list | `["migrate","run","--source","/usr/local/src/ferriskey/migrations"]` | Arguments for the database migrations job container. |
| databaseMigrations.backoffLimit | int | `nil` | Backoff limit for the database migrations job. |
| databaseMigrations.command | list | `["sqlx"]` | Command for the database migrations job container. |
| databaseMigrations.dnsConfig | object | `{}` | DNS config for the database migrations job workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| databaseMigrations.emptyDirSize | string | `nil` | EmptyDir size for the database migrations job. |
| databaseMigrations.env | list | `[]` | Environment variables for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| databaseMigrations.envFrom | list | `[]` | Environment variables from sources for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| databaseMigrations.ephemeralContainers | list | `[]` | Ephemeral containers for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ephemeralcontainer-v1-core |
| databaseMigrations.hostAliases | list | `[]` | Host aliases for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| databaseMigrations.hostIPC | bool | `nil` | Use host's IPC namespace for the database migrations job pods. |
| databaseMigrations.hostNetwork | bool | `nil` | Use host's network namespace for the database migrations job pods. |
| databaseMigrations.hostPID | bool | `nil` | Use host's PID namespace for the database migrations job pods. |
| databaseMigrations.hostUsers | bool | `nil` | Use host's user namespace for the database migrations job pods. |
| databaseMigrations.image.pullPolicy | string | `nil` | Pull policy for the image. |
| databaseMigrations.image.repository | string | `nil` | Repository for the image to use. Default to the API image repository. |
| databaseMigrations.image.tag | string | `nil` | Tag for the image to use. Default to the API image tag. |
| databaseMigrations.imagePullSecrets | list | `[]` | Image pull secrets for the database migrations job. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| databaseMigrations.initContainers | list | `[]` | Init containers for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| databaseMigrations.labels | object | `{}` | Labels on the database migrations job. |
| databaseMigrations.lifecycle | object | `{}` | Lifecycle for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#lifecycle-v1-core |
| databaseMigrations.nodeName | string | `nil` | Node name for the database migrations job pods. |
| databaseMigrations.nodeSelector | object | `{}` | Node selector for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#nodeselector-v1-core |
| databaseMigrations.podAnnotations | object | `{}` | Annotations on the database migrations job pods. |
| databaseMigrations.podLabels | object | `{}` | Labels on the database migrations job pods. |
| databaseMigrations.podSecurityContext | object | `{"fsGroup":1000,"runAsGroup":1000,"runAsNonRoot":true,"runAsUser":1000,"seccompProfile":{"type":"RuntimeDefault"}}` | Security context for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#podsecuritycontext-v1-core |
| databaseMigrations.preemptionPolicy | string | `nil` | Preemption policy for the database migrations job pods. |
| databaseMigrations.priority | int | `nil` | Priority for the database migrations job pods. |
| databaseMigrations.priorityClassName | string | `nil` | Priority class name for the database migrations job pods. |
| databaseMigrations.resources | object | `{"limits":{"memory":"512Mi"},"requests":{"memory":"128Mi"}}` | Resources for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#resourcerequirements-v1-core |
| databaseMigrations.runtimeClassName | string | `nil` | Runtime class name for the database migrations job pods. |
| databaseMigrations.schedulerName | string | `nil` | Scheduler name for the database migrations job pods. |
| databaseMigrations.securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true}` | Security context for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#securitycontext-v1-core |
| databaseMigrations.serviceAccount.annotations | object | `{}` | Annotations on the service account for the database migrations job pods. |
| databaseMigrations.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token for the database migrations job service account. |
| databaseMigrations.serviceAccount.create | bool | `true` | Create a service account for the database migrations job pods. |
| databaseMigrations.serviceAccount.labels | object | `{}` | Labels on the service account for the database migrations job pods. |
| databaseMigrations.serviceAccount.name | string | `nil` | Name of the service account for the database migrations job pods. Default is the database migrations job workload name. |
| databaseMigrations.shareProcessNamespace | bool | `nil` | Share a single process namespace between all of the containers for the database migrations job pods. |
| databaseMigrations.tolerations | list | `[]` | Tolerations for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| databaseMigrations.topologySpreadConstraints | list | `[]` | Topology spread constraints for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| databaseMigrations.ttlSecondsAfterFinished | int | `nil` | TTL seconds after finished for the database migrations job. If you're not using ArgoCD, set this to `0`. |
| databaseMigrations.volumeMounts | list | `[]` | Volume mounts for the database migrations job container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| databaseMigrations.volumes | list | `[]` | Volumes for the database migrations job pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| ingress.annotations | object | `{}` | Annotations on the ingress. |
| ingress.class | string | `nil` | Ingress class. |
| ingress.enabled | bool | `false` | Enable the ingress. |
| ingress.host | string | `nil` | Host for the ingress. |
| ingress.labels | object | `{}` | Labels on the ingress. |
| ingress.tls | list | `[]` | TLS configuration for the ingress. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ingresstls-v1-networking-k8s-io |
| nameOverride | string | `nil` | Override the name of the release. |
| postgresql.affinity | object | `{}` | Affinity for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| postgresql.annotations | object | `{}` | Annotations on PostgreSQL workloads. |
| postgresql.args | list | `[]` | Arguments for the PostgreSQL container. |
| postgresql.command | list | `[]` | Command for the PostgreSQL container. |
| postgresql.dnsConfig | object | `{}` | DNS config for the PostgreSQL workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| postgresql.emptyDirSize | string | `nil` | EmptyDir size for the PostgreSQL workload. Increase it if `postgresql.persistence` is disabled. |
| postgresql.enabled | bool | `true` | Enable the PostgreSQL. Not recommended to use in production. |
| postgresql.env | list | `[]` | Environment variables for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| postgresql.envFrom | list | `[]` | Environment variables from sources for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| postgresql.ephemeralContainers | list | `[]` | Ephemeral containers for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ephemeralcontainer-v1-core |
| postgresql.hostAliases | list | `[]` | Host aliases for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| postgresql.hostIPC | bool | `nil` | Use host's IPC namespace for the PostgreSQL pods. |
| postgresql.hostNetwork | bool | `nil` | Use host's network namespace for the PostgreSQL pods. |
| postgresql.hostPID | bool | `nil` | Use host's PID namespace for the PostgreSQL pods. |
| postgresql.hostUsers | bool | `nil` | Use host's user namespace for the PostgreSQL pods. |
| postgresql.hostname | string | `nil` | Hostname for the PostgreSQL pods. |
| postgresql.image.pullPolicy | string | `nil` | Pull policy for the image. |
| postgresql.image.repository | string | `"postgres"` | Repository for the image to use. |
| postgresql.image.tag | string | `"17"` | Tag for the image to use. Default to the chart's app version. |
| postgresql.imagePullSecrets | list | `[]` | Image pull secrets for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#localobjectreference-v1-core |
| postgresql.initContainers | list | `[]` | Init containers for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| postgresql.labels | object | `{}` | Labels on PostgreSQL workloads. |
| postgresql.lifecycle | object | `{}` | Lifecycle for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#lifecycle-v1-core |
| postgresql.livenessProbe | object | `{"failureThreshold":3,"initialDelaySeconds":30,"periodSeconds":10,"tcpSocket":{"port":"postgres"},"timeoutSeconds":5}` | Liveness probe for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| postgresql.nodeName | string | `nil` | Node name for the PostgreSQL pods. |
| postgresql.nodeSelector | object | `{}` | Node selector for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#nodeselector-v1-core |
| postgresql.persistence.accessModes | list | `["ReadWriteOnce"]` | Access modes for the PostgreSQL persistence. |
| postgresql.persistence.enabled | bool | `true` | Enable the PostgreSQL persistence. |
| postgresql.persistence.resources | object | `{"requests":{"storage":"5Gi"}}` | Resources for the PostgreSQL persistence. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumeresourcerequirements-v1-core |
| postgresql.persistence.storageClass | string | `nil` | Storage class for the PostgreSQL persistence. |
| postgresql.persistentVolumeClaimRetentionPolicy | object | `{}` | Persistent volume claim retention policy for the PostgreSQL workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#statefulsetpersistentvolumeclaimretentionpolicy-v1-apps |
| postgresql.podAnnotations | object | `{}` | Annotations on PostgreSQL pods. |
| postgresql.podLabels | object | `{}` | Labels on PostgreSQL pods. |
| postgresql.podSecurityContext | object | `{"fsGroup":999,"runAsGroup":999,"runAsNonRoot":true,"runAsUser":999,"seccompProfile":{"type":"RuntimeDefault"}}` | Security context for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#podsecuritycontext-v1-core |
| postgresql.preemptionPolicy | string | `nil` | Preemption policy for the PostgreSQL pods. |
| postgresql.priority | int | `nil` | Priority for the PostgreSQL pods. |
| postgresql.priorityClassName | string | `nil` | Priority class name for the PostgreSQL pods. |
| postgresql.readinessProbe | object | `{"failureThreshold":3,"initialDelaySeconds":5,"periodSeconds":5,"successThreshold":1,"tcpSocket":{"port":"postgres"},"timeoutSeconds":3}` | Readiness probe for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| postgresql.resources | object | `{"limits":{"memory":"256Mi"},"requests":{"memory":"128Mi"}}` | Resources for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#resourcerequirements-v1-core |
| postgresql.revisionHistoryLimit | int | `nil` | Revision history limit for the PostgreSQL workload. |
| postgresql.runtimeClassName | string | `nil` | Runtime class name for the PostgreSQL pods. |
| postgresql.schedulerName | string | `nil` | Scheduler name for the PostgreSQL pods. |
| postgresql.securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true}` | Security context for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#securitycontext-v1-core |
| postgresql.service.annotations | object | `{}` | Annotations on the service for the PostgreSQL pods. |
| postgresql.service.clusterIP | string | `"None"` | Cluster IP for the service for the PostgreSQL pods. |
| postgresql.service.clusterIPs | list | `[]` | Cluster IPs for the service for the PostgreSQL pods. |
| postgresql.service.externalIPs | list | `[]` | External IPs for the service for the PostgreSQL pods. |
| postgresql.service.externalName | string | `nil` | External name for the service for the PostgreSQL pods. |
| postgresql.service.externalTrafficPolicy | string | `nil` | ExternalTrafficPolicy for the service for the PostgreSQL pods. |
| postgresql.service.healthCheckNodePort | int | `nil` | Health check node port for the service for the PostgreSQL pods. |
| postgresql.service.internalTrafficPolicy | string | `nil` | InternalTrafficPolicy for the service for the PostgreSQL pods. |
| postgresql.service.ipFamilies | list | `[]` | IP families for the service for the PostgreSQL pods. |
| postgresql.service.ipFamilyPolicy | string | `nil` | IP family policy for the service for the PostgreSQL pods. |
| postgresql.service.labels | object | `{}` | Labels on the service for the PostgreSQL pods. |
| postgresql.service.loadBalancerClass | string | `nil` | Load balancer class for the service for the PostgreSQL pods. |
| postgresql.service.loadBalancerIP | string | `nil` | Load balancer IP for the service for the PostgreSQL pods. |
| postgresql.service.loadBalancerSourceRanges | list | `[]` | Load balancer source ranges for the service for the PostgreSQL pods. |
| postgresql.service.publishNotReadyAddresses | bool | `nil` | Publish not ready addresses for the service for the PostgreSQL pods. |
| postgresql.service.sessionAffinity | string | `nil` | Session affinity for the service for the PostgreSQL pods. |
| postgresql.service.sessionAffinityConfig | object | `{}` | Session affinity config for the service for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#sessionaffinityconfig-v1-core |
| postgresql.service.trafficDistribution | string | `nil` | Traffic distribution for the service for the PostgreSQL pods. |
| postgresql.service.type | string | `nil` | Type for the service for the PostgreSQL pods. |
| postgresql.serviceAccount.annotations | object | `{}` | Annotations on the service account for the PostgreSQL pods. |
| postgresql.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token for the PostgreSQL service account. |
| postgresql.serviceAccount.create | bool | `true` | Create a service account for the PostgreSQL pods. |
| postgresql.serviceAccount.labels | object | `{}` | Labels on the service account for the PostgreSQL pods. |
| postgresql.serviceAccount.name | string | `nil` | Name of the service account for the PostgreSQL pods. Default is the PostgreSQL workload name. |
| postgresql.setHostnameAsFQDN | bool | `nil` | Set hostname as FQDN for the PostgreSQL pods. |
| postgresql.shareProcessNamespace | bool | `nil` | Share a single process namespace between all of the containers for the PostgreSQL pods. |
| postgresql.subdomain | string | `nil` | Subdomain for the PostgreSQL pods. |
| postgresql.tolerations | list | `[]` | Tolerations for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| postgresql.topologySpreadConstraints | list | `[]` | Topology spread constraints for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| postgresql.updateStrategy | object | `{}` | Update strategy for the PostgreSQL workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#statefulsetupdatestrategy-v1-apps |
| postgresql.volumeMounts | list | `[]` | Volume mounts for the PostgreSQL container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| postgresql.volumes | list | `[]` | Volumes for the PostgreSQL pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |
| webapp.affinity | object | `{}` | Affinity for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#affinity-v1-core |
| webapp.annotations | object | `{}` | Annotations on webapp workloads. |
| webapp.api.protocol | string | `"https"` | Protocol for the API. Ignored if `webapp.api.url` is set. |
| webapp.api.url | string | `nil` | URL for the API. Default computed from the ingress configuration. |
| webapp.args | list | `[]` | Arguments for the webapp container. |
| webapp.command | list | `[]` | Command for the webapp container. |
| webapp.dnsConfig | object | `{}` | DNS config for the webapp workload. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#poddnsconfig-v1-core |
| webapp.emptyDirSize | string | `nil` | EmptyDir size for the webapp workload. |
| webapp.env | list | `[]` | Environment variables for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envvar-v1-core |
| webapp.envFrom | list | `[]` | Environment variables from sources for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#envfromsource-v1-core |
| webapp.ephemeralContainers | list | `[]` | Ephemeral containers for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#ephemeralcontainer-v1-core |
| webapp.hostAliases | list | `[]` | Host aliases for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#hostalias-v1-core |
| webapp.hostIPC | bool | `nil` | Use host's IPC namespace for the webapp pods. |
| webapp.hostNetwork | bool | `nil` | Use host's network namespace for the webapp pods. |
| webapp.hostPID | bool | `nil` | Use host's PID namespace for the webapp pods. |
| webapp.hostUsers | bool | `nil` | Use host's user namespace for the webapp pods. |
| webapp.hostname | string | `nil` | Hostname for the webapp pods. |
| webapp.image.pullPolicy | string | `nil` | Pull policy for the image. |
| webapp.image.repository | string | `"ghcr.io/ferriskey/ferriskey-webapp"` | Repository for the image to use. |
| webapp.image.tag | string | `nil` | Tag for the image to use. Default to the chart's app version. |
| webapp.imagePullSecrets | list | `[]` | Image pull secrets for the webapp pods. |
| webapp.initContainers | list | `[]` | Init containers for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#container-v1-core |
| webapp.labels | object | `{}` | Labels on webapp workloads. |
| webapp.lifecycle | object | `{}` | Lifecycle for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#lifecycle-v1-core |
| webapp.livenessProbe | object | `{"failureThreshold":3,"httpGet":{"path":"/","port":"http"},"initialDelaySeconds":30,"periodSeconds":10,"timeoutSeconds":5}` | Liveness probe for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| webapp.nodeName | string | `nil` | Node name for the webapp pods. |
| webapp.nodeSelector | object | `{}` | Node selector for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#nodeselector-v1-core |
| webapp.podAnnotations | object | `{}` | Annotations on webapp pods. |
| webapp.podLabels | object | `{}` | Labels on webapp pods. |
| webapp.podSecurityContext | object | `{"fsGroup":101,"runAsGroup":101,"runAsNonRoot":true,"runAsUser":101,"seccompProfile":{"type":"RuntimeDefault"}}` | Security context for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#podsecuritycontext-v1-core |
| webapp.preemptionPolicy | string | `nil` | Preemption policy for the webapp pods. |
| webapp.priority | int | `nil` | Priority for the webapp pods. |
| webapp.priorityClassName | string | `nil` | Priority class name for the webapp pods. |
| webapp.readinessProbe | object | `{"failureThreshold":3,"httpGet":{"path":"/","port":"http"},"initialDelaySeconds":5,"periodSeconds":5,"successThreshold":1,"timeoutSeconds":3}` | Readiness probe for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#probe-v1-core |
| webapp.replicas | int | `1` | Number of replicas for the webapp workload. |
| webapp.resources | object | `{"limits":{"memory":"64Mi"},"requests":{"memory":"32Mi"}}` | Resources for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#resourcerequirements-v1-core |
| webapp.revisionHistoryLimit | int | `nil` | Revision history limit for the webapp workload. |
| webapp.runtimeClassName | string | `nil` | Runtime class name for the webapp pods. |
| webapp.schedulerName | string | `nil` | Scheduler name for the webapp pods. |
| webapp.securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true}` | Security context for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#securitycontext-v1-core |
| webapp.service.annotations | object | `{}` | Annotations on the service for the webapp pods. |
| webapp.service.clusterIP | string | `nil` | Cluster IP for the service for the webapp pods. |
| webapp.service.clusterIPs | list | `[]` | Cluster IPs for the service for the webapp pods. |
| webapp.service.externalIPs | list | `[]` | External IPs for the service for the webapp pods. |
| webapp.service.externalName | string | `nil` | External name for the service for the webapp pods. |
| webapp.service.externalTrafficPolicy | string | `nil` | ExternalTrafficPolicy for the service for the webapp pods. |
| webapp.service.healthCheckNodePort | int | `nil` | Health check node port for the service for the webapp pods. |
| webapp.service.internalTrafficPolicy | string | `nil` | InternalTrafficPolicy for the service for the webapp pods. |
| webapp.service.ipFamilies | list | `[]` | IP families for the service for the webapp pods. |
| webapp.service.ipFamilyPolicy | string | `nil` | IP family policy for the service for the webapp pods. |
| webapp.service.labels | object | `{}` | Labels on the service for the webapp pods. |
| webapp.service.loadBalancerClass | string | `nil` | Load balancer class for the service for the webapp pods. |
| webapp.service.loadBalancerIP | string | `nil` | Load balancer IP for the service for the webapp pods. |
| webapp.service.loadBalancerSourceRanges | list | `[]` | Load balancer source ranges for the service for the webapp pods. |
| webapp.service.publishNotReadyAddresses | bool | `nil` | Publish not ready addresses for the service for the webapp pods. |
| webapp.service.sessionAffinity | string | `nil` | Session affinity for the service for the webapp pods. |
| webapp.service.sessionAffinityConfig | object | `{}` | Session affinity config for the service for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#sessionaffinityconfig-v1-core |
| webapp.service.trafficDistribution | string | `nil` | Traffic distribution for the service for the webapp pods. |
| webapp.service.type | string | `nil` | Type for the service for the webapp pods. |
| webapp.serviceAccount.annotations | object | `{}` | Annotations on the service account for the webapp pods. |
| webapp.serviceAccount.automountServiceAccountToken | bool | `nil` | Automount service account token for the webapp service account. |
| webapp.serviceAccount.create | bool | `true` | Create a service account for the webapp pods. |
| webapp.serviceAccount.labels | object | `{}` | Labels on the service account for the webapp pods. |
| webapp.serviceAccount.name | string | `nil` | Name of the service account for the webapp pods. Default is the webapp workload name. |
| webapp.setHostnameAsFQDN | bool | `nil` | Set hostname as FQDN for the webapp pods. |
| webapp.shareProcessNamespace | bool | `nil` | Share a single process namespace between all of the containers for the webapp pods. |
| webapp.subdomain | string | `nil` | Subdomain for the webapp pods. |
| webapp.tolerations | list | `[]` | Tolerations for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#toleration-v1-core |
| webapp.topologySpreadConstraints | list | `[]` | Topology spread constraints for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#topologyspreadconstraint-v1-core |
| webapp.volumeMounts | list | `[]` | Volume mounts for the webapp container. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volumemount-v1-core |
| webapp.volumes | list | `[]` | Volumes for the webapp pods. https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.33/#volume-v1-core |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.14.2](https://github.com/norwoodj/helm-docs/releases/v1.14.2)

toto
