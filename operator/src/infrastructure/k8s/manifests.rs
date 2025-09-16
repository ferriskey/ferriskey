use std::{collections::BTreeMap, vec};

use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Container, PodSpec, PodTemplateSpec, Service, ServicePort, ServiceSpec},
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::api::ObjectMeta;

use crate::domain::cluster::ClusterSpec;

pub fn make_deployment(spec: &ClusterSpec, namespace: &str) -> Deployment {
    Deployment {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(spec.replicas as i32),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([("app".to_string(), spec.name.clone())])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([(
                        "app".to_string(),
                        "ferriskey-api".to_string(),
                    )])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "ferriskey-api".into(),
                        image: Some(format!("ghcr.io/ferriskey/ferriskey-api:{}", spec.version)),
                        env: Some(vec![k8s_openapi::api::core::v1::EnvVar {
                            name: "PORT".to_string(),
                            value: Some("3333".to_string()),
                            ..Default::default()
                        }]),
                        ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 3333,
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

pub fn api_service(spec: &ClusterSpec, namespace: &str) -> Service {
    Service {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-api-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(BTreeMap::from([(
                "app".to_string(),
                "ferriskey-api".to_string(),
            )])),
            ports: Some(vec![ServicePort {
                port: 3333,
                target_port: Some(
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(3333),
                ),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        status: None,
    }
}
