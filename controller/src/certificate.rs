use crate::cert_issuer::{self, CertIssuer};
use crate::consts::labels::{
    CACHED, CACHE_NAME, CERT_ISSUER, CERT_NAME, MANAGED_BY_KEY, MANAGED_BY_VALUE, TLS,
};
use crate::error::Error;
use crate::store::Store;
use futures::stream::{FuturesUnordered, StreamExt};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::ByteString;
use kube::api::Meta;
use kube::api::{Api, PostParams};
use std::collections::BTreeMap;
use uuid::Uuid;

pub type Certificate = Secret;

pub async fn cache_and_create_for_namespaces(
    store: &Store,
    cert_issuer: &CertIssuer,
) -> Result<Vec<Certificate>, Error> {
    match create_cache(&store, &cert_issuer).await {
        Ok(cached_cert) => {
            let created_certificates = cert_issuer
                .spec
                .namespaces
                .iter()
                .map(|ns| create_from_cache(store, &cached_cert, &ns))
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<Certificate>>();

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

    let contents: BTreeMap<String, ByteString> =
        vec![("tls.crt", "tls.crt.data"), ("tls.key", "tls.key.data")]
            .iter()
            .map(|(key, value)| (key.to_string(), ByteString(value.as_bytes().to_vec())))
            .collect();

    let labels: BTreeMap<String, String> = vec![
        (MANAGED_BY_KEY, MANAGED_BY_VALUE),
        (CERT_ISSUER, &cert_issuer_name),
        (CERT_NAME, &cert_name),
        (CACHED, "true"),
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
        data: Some(contents),
        ..Default::default()
    };

    let cert_api =
        Api::<Certificate>::namespaced(store.get_client().clone(), store.get_namespace());
    let pp = PostParams::default();

    Ok(cert_api.create(&pp, &cert).await?)
}

async fn create_from_cache(store: &Store, secret: &Secret, ns: &str) -> Result<Certificate, Error> {
    let secret = secret.clone();

    let mut labels = secret
        .clone()
        .metadata
        .labels
        .unwrap_or_else(|| BTreeMap::new());

    labels.remove(CACHED);
    labels.insert(CACHE_NAME.to_string(), secret.clone().name());

    let name = labels
        .get(CERT_NAME)
        .map(|c| c.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let cert = Certificate {
        metadata: ObjectMeta {
            name: Some(name),
            namespace: Some(ns.to_string()),
            owner_references: secret.metadata.owner_references,
            labels: Some(labels),
            ..ObjectMeta::default()
        },
        type_: Some(TLS.to_string()),
        data: secret.data,
        ..Default::default()
    };

    let cert_api = Api::<Certificate>::namespaced(store.get_client().clone(), ns);
    let pp = PostParams::default();

    Ok(cert_api.create(&pp, &cert).await?)
}
