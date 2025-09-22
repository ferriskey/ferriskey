use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Container, ContainerPort, EnvVar, PodSpec, PodTemplateSpec, Service, ServicePort,
            ServiceSpec,
        },
    },
    apimachinery::pkg::{apis::meta::v1::LabelSelector, util::intstr::IntOrString},
};
use kube::api::ObjectMeta;

use crate::domain::cluster::ClusterSpec;

pub mod api;

pub fn make_webapp_deployment(spec: &ClusterSpec, namespace: &str) -> Deployment {
    let app_label = format!("ferriskey-webapp-{}", spec.name);

    Deployment {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-webapp-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(spec.replicas as i32),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([("app".to_string(), app_label.clone())])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([("app".to_string(), app_label.clone())])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "ferriskey-webapp".to_string(),
                        image: Some(format!(
                            "ghcr.io/ferriskey/ferriskey-webapp:{}",
                            spec.version
                        )),
                        env: Some(vec![EnvVar {
                            name: "API_URL".to_string(),
                            value: Some(spec.api.api_url.clone()),
                            ..Default::default()
                        }]),
                        ports: Some(vec![ContainerPort {
                            container_port: 80,
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        status: None,
    }
}

pub fn make_webapp_service(spec: &ClusterSpec, namespace: &str) -> Service {
    let app_label = format!("ferriskey-webapp-{}", spec.name);

    Service {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-webapp-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(BTreeMap::from([("app".to_string(), app_label)])),
            ports: Some(vec![ServicePort {
                port: 80,
                target_port: Some(IntOrString::Int(80)),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        status: None,
    }
}
