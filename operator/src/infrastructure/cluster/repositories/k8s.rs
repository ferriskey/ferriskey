use k8s_openapi::api::{
    apps::v1::Deployment,
    batch::v1::Job,
    core::v1::{Secret, Service},
};
use kube::{
    Api, Client,
    api::{Patch, PatchParams},
};

use crate::{
    domain::{
        cluster::{ClusterPort, ClusterSpec, ClusterStatus},
        error::OperatorError,
    },
    infrastructure::cluster::manifests::{
        api::{
            deployment::make_deployment, job::make_migration_job, secret::make_admin_secret,
            service::make_api_service,
        },
        make_webapp_deployment, make_webapp_service,
    },
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
        let secrets: Api<Secret> = Api::namespaced(self.client().clone(), namespace);
        let jobs: Api<Job> = Api::namespaced(self.client().clone(), namespace);

        let secret = make_admin_secret(spec, namespace);

        secrets
            .patch(
                &secret.metadata.name.clone().unwrap(),
                &PatchParams::apply("ferriskey-operator"),
                &Patch::Apply(&secret),
            )
            .await
            .map_err(|e| OperatorError::ApplyApiError {
                message: e.to_string(),
            })?;

        let job = make_migration_job(spec, namespace);

        jobs.patch(
            &job.metadata.name.clone().unwrap(),
            &PatchParams::apply("ferriskey-operator"),
            &Patch::Apply(&job),
        )
        .await
        .map_err(|e| OperatorError::ApplyApiError {
            message: e.to_string(),
        })?;

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

        let svc = make_api_service(spec, namespace);

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

        let webapp_deployment = make_webapp_deployment(spec, namespace);
        let webapp_service = make_webapp_service(spec, namespace);

        deployments
            .patch(
                &webapp_deployment.metadata.name.clone().unwrap(),
                &PatchParams::apply("ferriskey-operator"),
                &Patch::Apply(&webapp_deployment),
            )
            .await
            .map_err(|e| OperatorError::ApplyApiError {
                message: e.to_string(),
            })?;

        services
            .patch(
                &webapp_service.metadata.name.clone().unwrap(),
                &PatchParams::apply("ferriskey-operator"),
                &Patch::Apply(&webapp_service),
            )
            .await
            .map_err(|e| OperatorError::ApplyApiError {
                message: e.to_string(),
            })?;

        Ok(ClusterStatus {
            ready: true,
            message: Some("Cluster applied successfully".into()),
            phase: Some("Progressing".to_string()),
        })
    }

    async fn delete(&self, spec: &ClusterSpec, namespace: &str) -> Result<(), OperatorError> {
        let deployments: Api<Deployment> = Api::namespaced(self.client().clone(), namespace);
        let services: Api<Service> = Api::namespaced(self.client().clone(), namespace);
        let jobs: Api<Job> = Api::namespaced(self.client().clone(), namespace);
        let secrets: Api<Secret> = Api::namespaced(self.client().clone(), namespace);

        let deployment_name = format!("ferriskey-api-{}", spec.name);
        let service_name = format!("ferriskey-api-{}", spec.name);
        let secret_name = format!("ferriskey-admin-{}", spec.name);
        let job_name = format!("ferriskey-migrations-{}", spec.name);
        let webapp_name = format!("ferriskey-webapp-{}", spec.name);

        match deployments
            .delete(&deployment_name, &Default::default())
            .await
        {
            Ok(_) => tracing::info!("✅ Deployment {} supprimé", deployment_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Deployment {} déjà supprimé (404)", deployment_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du deployment {}: {}",
                    deployment_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Deployment deletion error: {}", e),
                });
            }
        }

        // Suppression du service avec gestion des erreurs 404
        match services.delete(&service_name, &Default::default()).await {
            Ok(_) => tracing::info!("✅ Service {} supprimé", service_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Service {} déjà supprimé (404)", service_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du service {}: {}",
                    service_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Service deletion error: {}", e),
                });
            }
        }

        match secrets.delete(&secret_name, &Default::default()).await {
            Ok(_) => tracing::info!("✅ Secret {} supprimé", secret_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Secret {} déjà supprimé (404)", secret_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du secret {}: {}",
                    secret_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Secret deletion error: {}", e),
                });
            }
        }

        match jobs.delete(&job_name, &Default::default()).await {
            Ok(_) => tracing::info!("✅ Job {} supprimé", job_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Job {} déjà supprimé (404)", job_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du job {}: {}",
                    job_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Job deletion error: {}", e),
                });
            }
        }

        match deployments.delete(&webapp_name, &Default::default()).await {
            Ok(_) => tracing::info!("✅ Webapp Deployment {} supprimé", webapp_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Webapp Deployment {} déjà supprimé (404)", webapp_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du webapp deployment {}: {}",
                    webapp_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Webapp Deployment deletion error: {}", e),
                });
            }
        }

        match services.delete(&webapp_name, &Default::default()).await {
            Ok(_) => tracing::info!("✅ Webapp Service {} supprimé", webapp_name),
            Err(kube::Error::Api(api_error)) if api_error.code == 404 => {
                tracing::info!("ℹ️ Webapp Service {} déjà supprimé (404)", webapp_name);
            }
            Err(e) => {
                tracing::error!(
                    "❌ Erreur lors de la suppression du webapp service {}: {}",
                    webapp_name,
                    e
                );
                return Err(OperatorError::DeleteApiError {
                    message: format!("Webapp Service deletion error: {}", e),
                });
            }
        }

        tracing::info!("🎉 Cleanup terminé avec succès");
        Ok(())
    }
}
