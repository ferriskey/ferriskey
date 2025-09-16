use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};
use kube::{
    Api, Client,
    api::{Patch, PatchParams},
};

use crate::{
    domain::{
        cluster::{ClusterPort, ClusterSpec, ClusterStatus},
        error::OperatorError,
    },
    infrastructure::k8s::manifests::{api_service, make_deployment},
};

#[derive(Clone)]
pub struct K8sClusterRepository {
    client: Client,
}

impl K8sClusterRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn client(&self) -> Client {
        self.client.clone()
    }
}

impl ClusterPort for K8sClusterRepository {
    async fn apply(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<ClusterStatus, OperatorError> {
        let deployments: Api<Deployment> = Api::namespaced(self.client().clone(), namespace);
        let services: Api<Service> = Api::namespaced(self.client().clone(), namespace);

        let dep = make_deployment(spec, namespace);
        deployments
            .patch(
                &dep.metadata.name.clone().unwrap(),
                &PatchParams::apply("ferriskey-operator"),
                &Patch::Apply(&dep),
            )
            .await
            .map_err(|e| OperatorError::ApplyApiError {
                message: e.to_string(),
            })?;

        let svc = api_service(spec, namespace);

        services
            .patch(
                &svc.metadata.name.clone().unwrap(),
                &PatchParams::apply("ferriskey-operator"),
                &Patch::Apply(&svc),
            )
            .await
            .map_err(|e| OperatorError::ApplyApiError {
                message: e.to_string(),
            })?;

        Ok(ClusterStatus {
            ready: true,
            message: Some("Cluster applied successfully".into()),
        })
    }

    async fn delete(&self, spec: &ClusterSpec, namespace: &str) -> Result<(), OperatorError> {
        let deployments: Api<Deployment> = Api::namespaced(self.client().clone(), namespace);
        let services: Api<Service> = Api::namespaced(self.client().clone(), namespace);

        let name = format!("ferriskey-api-{}", spec.name);

        deployments
            .delete(&name, &Default::default())
            .await
            .map_err(|e| OperatorError::DeleteApiError {
                message: e.to_string(),
            })?;

        services
            .delete(&name, &Default::default())
            .await
            .map_err(|e| OperatorError::DeleteApiError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}
