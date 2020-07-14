use anyhow;
use kube_derive::CustomResource;
use log::info;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(apiextensions = "v1beta1")]
#[kube(group = "certmaster.kuberails.com", version = "v1", namespaced)]
pub struct CertIssuerSpec {
    domain_name: String,
    dns_provider: DnsProviderSpec,
    secret_name: Option<String>,
    namespaces: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DnsProviderSpec {
    provider: DnsProvider,
    key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum DnsProvider {
    DigitalOcean,
    Cloudflare,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let crd = CertIssuer::crd();

    info!(
        "Creating CertIssuer CRD:\n\n{}\n",
        serde_yaml::to_string(&crd)?
    );

    Ok(())
}
