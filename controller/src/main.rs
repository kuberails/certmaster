use anyhow::anyhow;
use chrono::prelude::*;
use futures::StreamExt;
use kube::{
    api::{Api, WatchEvent},
    runtime::Informer,
    Client,
};
use kube_derive::CustomResource;
use log::{info, warn};
use rweb::Schema;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, Schema)]
#[kube(group = "certmaster.kuberails.com", version = "v1", namespaced)]
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

#[derive(Serialize, Deserialize, Clone, Debug, Schema)]
#[serde(rename_all = "camelCase")]
struct DnsProviderSpec {
    provider: DnsProvider,
    key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Schema)]
enum DnsProvider {
    #[serde(rename = "digtalocean")]
    DigitalOcean,
    #[serde(rename = "cloudflare")]
    Cloudflare,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;

    let mut cert_issuers: Api<CertIssuer> = Api::all(client.clone());
    let mut inform = Informer::new(cert_issuers);

    loop {
        match poll_events(&inform).await {
            Ok(event) => {
                handle(event).await;
            }
            e => {
                warn!("{:#?}", e);
                inform.set_version(Utc::now().timestamp_millis().to_string());
                cert_issuers = Api::all(client.clone());
                inform = Informer::new(cert_issuers);
            }
        }
    }
}

async fn poll_events(inform: &Informer<CertIssuer>) -> anyhow::Result<WatchEvent<CertIssuer>> {
    let mut events = inform.poll().await?.boxed();

    match events.next().await {
        Some(Ok(event)) => Ok(event),
        e => {
            events.next().await;
            Err(anyhow!("ERROR {:#?}", e))
        }
    }
}

async fn handle(event: WatchEvent<CertIssuer>) -> () {
    match &event {
        WatchEvent::Added(crd) => println!("Added CRD: {:#?}", crd),
        WatchEvent::Modified(crd) => println!("Modified CRD: {:#?}", crd),
        WatchEvent::Deleted(crd) => println!("Deleted CRD: {:#?}", crd),
        WatchEvent::Error(e) => println!("Error event: {:?}", e),
        something_else => info!("WatchEvent unmatched event: {:#?}", something_else),
    };
}
