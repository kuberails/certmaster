use crate::error::Error;
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
    pub dns_provider: DnsProviderSpec,
    pub secret_name: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespaces: Vec<String>,
}

fn default_namespace() -> Vec<String> {
    vec!["default".to_string()]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DnsProviderSpec {
    pub provider: DnsProvider,
    pub key: String,
    pub secret_key: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DnsProvider {
    #[serde(rename = "digtalocean")]
    DigitalOcean,
    #[serde(rename = "cloudflare")]
    Cloudflare,
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
