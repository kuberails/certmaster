use anyhow;
use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(apiextensions = "v1beta1")] // remove this once schemas is added
#[kube(group = "certmaster.kuberails.com", version = "v1", namespaced)]
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
struct DnsProviderSpec {
    provider: DnsProvider,
    key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum DnsProvider {
    DigitalOcean,
    Cloudflare,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let crd = CertIssuer::crd();

    println!("\n{}\n", serde_yaml::to_string(&crd)?);

    Ok(())
}
