use crate::error::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::Resource;
use kube::api::Meta;
use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(group = "certmaster.kuberails.com", version = "v1")]
#[serde(rename_all = "camelCase")]
pub struct CertIssuerSpec {
    domain_name: String,
    dns_provider: DnsProviderSpec,
    secret_name: Option<String>,
    #[serde(default = "default_namespace")]
    namespaces: Vec<String>,
}

fn default_namespace() -> Vec<String> {
    vec!["default".to_string()]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct DnsProviderSpec {
    provider: DnsProvider,
    key: String,
    secret_key: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
enum DnsProvider {
    #[serde(rename = "digtalocean")]
    DigitalOcean,
    #[serde(rename = "cloudflare")]
    Cloudflare,
}

pub fn owner_reference(cert_issuer: CertIssuer) -> Result<OwnerReference, Error> {
    let meta = cert_issuer.meta().clone();

    Ok(OwnerReference {
        api_version: CertIssuer::API_VERSION.to_string(),
        kind: CertIssuer::KIND.to_string(),
        name: meta
            .name
            .ok_or(Error::MissingObjectKey(".metadata.name"))?
            .to_string(),
        uid: meta
            .uid
            .ok_or(Error::MissingObjectKey(".metadata.backtrace"))?
            .to_string(),
        ..OwnerReference::default()
    })
}