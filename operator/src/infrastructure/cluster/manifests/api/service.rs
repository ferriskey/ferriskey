use std::collections::BTreeMap;

use k8s_openapi::{
    api::core::v1::{Service, ServicePort, ServiceSpec},
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::api::ObjectMeta;

use crate::domain::cluster::ClusterSpec;

pub fn make_api_service(spec: &ClusterSpec, namespace: &str) -> Service {
    let app_label = format!("ferriskey-api-{}", spec.name);

    Service {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-api-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(BTreeMap::from([(
                "app".to_string(),
                app_label, // ← Correction ici aussi
            )])),
            ports: Some(vec![ServicePort {
                port: 3333,
                target_port: Some(IntOrString::Int(3333)),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        status: None,
    }
}
