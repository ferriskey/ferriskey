use crate::crd::cluster::FerriskeyCluster;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{ApiResource, DeleteParams, PostParams};
use kube::core::DynamicObject;
use kube::{Api, Client, Resource, ResourceExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CNPGClusterSpec {
    instances: u32,
    image_name: String,
    storage: CNPGStorage,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CNPGStorage {
    storage_class: String,
    size: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CPNGCluster {
    api_version: String,
    kind: String,
    metadata: ObjectMeta,
    spec: CNPGClusterSpec,
}

pub async fn reconcile_postgres(
    cluster: &FerriskeyCluster,
    client: &Client,
) -> Result<(), kube::Error> {
    let ns = cluster.metadata.namespace.as_deref().unwrap_or("default");
    let name = format!("{}-postgres", cluster.name_any());

    let cluster_ar = ApiResource::from_gvk_with_plural(
        &kube::core::gvk::GroupVersionKind::gvk("postgresql.cnpg.io", "v1", "Cluster"),
        "clusters", // ⚠️ Doit être le pluriel défini dans la CRD CNPG
    );

    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &cluster_ar);

    if cluster.meta().deletion_timestamp.is_some() {
        if api.get_opt(&name).await?.is_some() {
            api.delete(&name, &DeleteParams::default()).await.ok();
            info!("🗑️ Cluster PostgreSQL '{}' supprimé", name);
        }

        return Ok(());
    }

    if api.get_opt(&name).await?.is_none() {
        let raw = json!({
            "apiVersion": "postgresql.cnpg.io/v1",
            "kind": "Cluster",
            "metadata": {
                "name": name,
                "namespace": ns
            },
            "spec": {
                "instances": 1,
                "primaryUpdateStrategy": "unsupervised",
                "imageName": "ghcr.io/cloudnative-pg/postgresql:16",
                "storage": {
                    "storageClass": "standard",
                    "size": "1Gi"
                }
            }
        });

        let dyn_obj: DynamicObject = serde_json::from_value(raw).unwrap();
        api.create(&PostParams::default(), &dyn_obj).await?;
        info!("🐘 Cluster PostgreSQL '{}' créé", name);
    } else {
        info!("🔁 Cluster PostgreSQL '{}' déjà présent", name);
    }

    Ok(())
}
