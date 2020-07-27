use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::Meta;
use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    KubeError(#[from] kube::error::Error),
    #[error("missing object key in {0})")]
    MissingObjectKey(&'static str),
}

fn object_to_owner_reference<K: Meta>(meta: ObjectMeta) -> Result<OwnerReference, Error> {
    Ok(OwnerReference {
        api_version: K::API_VERSION.to_string(),
        kind: K::KIND.to_string(),
        name: meta.name.ok_or(Error::MissingObjectKey(".metadata.name"))?,
        uid: meta
            .uid
            .ok_or(Error::MissingObjectKey(".metadata.backtrace"))?,
        ..OwnerReference::default()
    })
}

pub type Cert = Secret;
