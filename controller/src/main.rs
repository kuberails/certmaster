use futures::prelude::*;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::{
    api::{Api, ListParams, Meta},
    Client,
};
use kube_derive::CustomResource;
use kube_runtime::controller::{Context, Controller, ReconcilerAction};
use kube_runtime::watcher;
use log::{info, warn};
use rweb::Schema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio;
use tokio::task;
use tokio::time::Duration;

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

#[derive(Debug, Error)]
enum Error {
    #[error("missing object key in {0})")]
    MissingObjectKey(&'static str),
}

struct Data {
    client: Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;

    let cert_issuers: Api<CertIssuer> = Api::all(client.clone());
    let cert_issuers2 = cert_issuers.clone();

    let certs: Api<Secret> = Api::all(client.clone());

    let controller_task = task::spawn(async {
        Controller::new(cert_issuers, ListParams::default().include_uninitialized())
            .owns(certs, ListParams::default())
            .run(reconcile, error_policy, Context::new(Data { client }))
            .for_each(|res| async move {
                match res {
                    Ok(o) => info!("reconciled {:?}", o),
                    Err(e) => warn!("reconcile failed: {}", e),
                }
            })
            .await;
    });

    let watcher_task = task::spawn(async {
        let watcher = watcher(cert_issuers2, ListParams::default());
        let _ = watcher
            .try_for_each(|event| async move {
                match event {
                    watcher::Event::Deleted(cert_issuer) => handle_delete(cert_issuer),
                    _event => (),
                }
                Ok(())
            })
            .await;
    });

    let _ = futures::join!(controller_task, watcher_task);

    Ok(())
}

async fn reconcile(cert_issuer: CertIssuer, ctx: Context<Data>) -> Result<ReconcilerAction, Error> {
    info!("Cert Issuer Reconciled: {:#?}", cert_issuer);
    // let client = ctx.get_ref().client.clone();

    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(30)),
    })
}

fn handle_delete(cert_issuer: CertIssuer) {
    info!("Cert Issuer deleted: {:?}", cert_issuer)
}

fn error_policy(error: &Error, _ctx: Context<Data>) -> ReconcilerAction {
    warn!("Error policy triggered: {:#?}", error);

    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(1)),
    }
}

// fn object_to_owner_reference<K: Meta>(meta: ObjectMeta) -> Result<OwnerReference, Error> {
//     Ok(OwnerReference {
//         api_version: K::API_VERSION.to_string(),
//         kind: K::KIND.to_string(),
//         name: meta.name.ok_or(Error::MissingObjectKey(".metadata.name"))?,
//         uid: meta
//             .uid
//             .ok_or(Error::MissingObjectKey(".metadata.backtrace"))?,
//         ..OwnerReference::default()
//     })
// }
