use anyhow;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube_derive::CustomResource;
use rweb::openapi::Entity;
use rweb::openapi::Schema;
use rweb::Schema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, Schema)]
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

#[derive(Serialize, Deserialize, Clone, Debug, Schema)]
#[serde(tag = "provider")]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    DigitalOcean(BasicAuth),
    Cloudflare(BasicAuth),
}

#[derive(Serialize, Deserialize, Clone, Debug, Schema)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuth {
    key: String,
    secret_key: String,
}

fn main() -> anyhow::Result<()> {
    let spec = CertIssuerSpec {
        domain_name: "praveenperera.com".to_string(),
        dns_provider: DnsProvider::DigitalOcean(BasicAuth {
            key: "key".to_string(),
            secret_key: "secretKey".to_string(),
        }),
        secret_name: Some("secret+name".to_string()),
        namespaces: vec!["default".to_string()],
    };

    let crd = CertIssuer::crd();
    let schema = <CertIssuerSpec as Entity>::describe();

    println!("CRD: \n{}\n", serde_yaml::to_string(&crd)?);
    println!("\nSCHEMA: \n{}\n", serde_yaml::to_string(&schema)?);
    println!("\nSPEC: \n{}\n", serde_yaml::to_string(&spec)?);

    Ok(())
}
