pub mod dns_provider;

use crate::error::Error;
use dns_provider::DnsProvider;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use k8s_openapi::Resource;
use kube::api::Meta;
use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(group = "certmaster.kuberails.com", version = "v1")]
#[serde(rename_all = "camelCase")]
pub struct CertIssuerSpec {
    pub domain_name: String,
    pub dns_provider: DnsProvider,
    pub secret_name: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespaces: Vec<String>,
}

fn default_namespace() -> Vec<String> {
    vec!["default".to_string()]
}

pub fn owner_reference(cert_issuer: &CertIssuer) -> Result<OwnerReference, Error> {
    Ok(OwnerReference {
        api_version: CertIssuer::API_VERSION.to_string(),
        kind: CertIssuer::KIND.to_string(),
        name: cert_issuer
            .meta()
            .name
            .clone()
            .ok_or(Error::MissingObjectKey(".metadata.name"))?
            .to_string(),
        uid: cert_issuer
            .meta()
            .uid
            .clone()
            .ok_or(Error::MissingObjectKey(".metadata.backtrace"))?
            .to_string(),
        ..OwnerReference::default()
    })
}
