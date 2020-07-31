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
    pub dns_provider: DnsProvider,
    pub secret_name: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespaces: Vec<String>,
}

fn default_namespace() -> Vec<String> {
    vec!["default".to_string()]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "provider")]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    DigitalOcean(BasicAuth),
    Cloudflare(BasicAuth),
    Route53(Route53),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuth {
    key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Route53 {
    access_key: String,
    secret_access_key: String,
    region: String,
    profile: Option<String>,
    hosted_zone_id: Option<String>,
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
