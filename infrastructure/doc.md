## Launch Local K8S Cluster

```bash
k3d cluster create ferriskey \
  --registry-create ferriskey-registry:0.0.0.0:5000 \
  --port 8080:80@loadbalancer \
  --agents 3
````
