use crate::cert_issuer::{self, CertIssuer};
use crate::error::Error;
use crate::store::Store;
use futures::future::{self, Future};
use futures::stream::{FuturesUnordered, StreamExt};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::ByteString;
use kube::api::Meta;
use kube::api::{Api, PostParams};
use std::collections::BTreeMap;
use uuid::Uuid;

pub type Certificate = Secret;

async fn cache_and_create_for_namespaces(
    store: &Store,
    cert_issuer: &CertIssuer,
) -> Result<Vec<Certificate>, Error> {
    match create_cache(&store, &cert_issuer).await {
        Ok(cached_cert) => {
            let created_certificates: Vec<Certificate> = cert_issuer
                .spec
                .namespaces
                .iter()
                .map(|ns| create_from_cached_cert(store, &cached_cert, &ns))
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect();

            Ok(created_certificates)
        }
        Err(error) => Err(error),
    }
}

async fn create_cache(store: &Store, cert_issuer: &CertIssuer) -> Result<Certificate, Error> {
    let cert_issuer = cert_issuer.clone();
    let cert_issuer_name = cert_issuer
        .meta()
        .name
        .as_ref()
        .ok_or(Error::MissingObjectKey(".meta.name"))?
        .to_string();

    let cert_name = cert_issuer
        .clone()
        .spec
        .secret_name
        .unwrap_or_else(|| cert_issuer_name.clone());

    let cache_name = Uuid::new_v4().to_string();

    let contents: BTreeMap<String, ByteString> = vec![(
        "content".to_string(),
        ByteString("hello".as_bytes().to_vec()),
    )]
    .into_iter()
    .collect();

    let labels: BTreeMap<String, String> = vec![
        (
            "app.kubernetes.io/managed-by",
            "certmaster.kuberails.com/v1",
        ),
        ("certmaster.kuberails.com/certIssuer", &cert_issuer_name),
        ("certmaster.kuberails.com/certName", &cert_name),
        ("certmaster.kuberails.com/cached", "true"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect();

    let cert = Certificate {
        metadata: ObjectMeta {
            name: Some(cache_name),
            namespace: Some(store.get_namespace().to_string()),
            owner_references: Some(vec![OwnerReference {
                controller: Some(true),
                ..cert_issuer::owner_reference(cert_issuer)?
            }]),
            labels: Some(labels),
            ..ObjectMeta::default()
        },
        // type_: Some("kubernetes.io/tls".to_string()),
        data: Some(contents),
        ..Default::default()
    };

    let cert_api =
        Api::<Certificate>::namespaced(store.get_client().clone(), store.get_namespace());
    let pp = PostParams::default();

    Ok(cert_api.create(&pp, &cert).await?)
}

async fn create_from_cached_cert(
    store: &Store,
    secret: &Secret,
    ns: &str,
) -> Result<Certificate, Error> {
    let secret = secret.clone();

    let mut labels = secret
        .clone()
        .metadata
        .labels
        .unwrap_or_else(|| BTreeMap::new());

    let name = labels
        .get("certmaster.kuberails.com/certName")
        .map(|c| c.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    labels.remove("certmaster.kuberails.com/cached");
    labels.insert(
        "certmaster.kuberails.com/cacheName".to_string(),
        secret.clone().name(),
    );

    let cert = Certificate {
        metadata: ObjectMeta {
            name: Some(name),
            namespace: Some(ns.to_string()),
            owner_references: secret.metadata.owner_references,
            labels: Some(labels),
            ..ObjectMeta::default()
        },
        type_: Some("kubernetes.io/tls".to_string()),
        data: secret.data,
        ..Default::default()
    };

    let cert_api = Api::<Certificate>::namespaced(store.get_client().clone(), ns);
    let pp = PostParams::default();

    Ok(cert_api.create(&pp, &cert).await?)
}
