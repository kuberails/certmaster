use crate::cert_issuer::{self, CertIssuer};
use crate::error::Error;
use crate::store::Store;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::ByteString;
use kube::api::Meta;
use kube::api::{Api, PostParams};
use std::collections::BTreeMap;

pub type Certificate = Secret;

pub async fn create(store: Store, cert_issuer: CertIssuer) -> Result<Certificate, Error> {
    let mut contents = BTreeMap::new();
    contents.insert(
        "content".to_string(),
        ByteString("hello".as_bytes().to_vec()),
    );

    let name = cert_issuer
        .meta()
        .name
        .as_ref()
        .ok_or(Error::MissingObjectKey(".meta.name"))?
        .to_string();

    let labels: BTreeMap<String, String> = vec![
        (
            "manager".to_string(),
            "certmaster.kuberails.com".to_string(),
        ),
        (
            "certmaster.kuberails.com/certIssuer".to_string(),
            name.clone(),
        ),
    ]
    .into_iter()
    .collect();

    let cert = Certificate {
        metadata: ObjectMeta {
            name: Some(format!("{}-cert", name)),
            namespace: Some("avencera".to_string()),
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

    let cert_api = Api::<Certificate>::namespaced(store.get_client().clone(), "avencera");
    let pp = PostParams::default();

    Ok(cert_api.create(&pp, &cert).await?)
}