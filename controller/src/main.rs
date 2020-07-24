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
use tokio::{sync::RwLock, time::Duration};

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

type State = Context<RwLock<InnerState>>;

#[derive(Debug)]
struct InnerState {
    cert_issuers: Vec<CertIssuer>,
    certs: Vec<Secret>,
}

impl InnerState {
    fn new() -> InnerState {
        InnerState {
            cert_issuers: vec![],
            certs: vec![],
        }
    }
}

async fn add_cert_issuer(state: &State, cert_issuer: CertIssuer) -> () {
    state.get_ref().write().await.cert_issuers.push(cert_issuer);
}

async fn delete_cert_issuer(state: &State, cert_issuer: CertIssuer) -> () {
    state
        .get_ref()
        .write()
        .await
        .cert_issuers
        .retain(|ci| ci.meta().name != cert_issuer.meta().name);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;

    // for controller
    let cert_issuer: Api<CertIssuer> = Api::all(client.clone());
    let context = Context::new(RwLock::new(InnerState::new()));

    // for watcher
    let cert_issuer_clone = cert_issuer.clone();
    let context_clone = context.clone();

    let certs: Api<Secret> = Api::all(client.clone());

    let controller_task = task::spawn(async move {
        Controller::new(cert_issuer, ListParams::default().include_uninitialized())
            .owns(certs, ListParams::default())
            .run(reconcile, error_policy, context)
            .for_each(|res| async move {
                match res {
                    Ok(o) => info!("reconciled {:?}", o),
                    Err(e) => warn!("reconcile failed: {}", e),
                }
            })
            .await;
    });

    let watcher_task = task::spawn(async move {
        let watcher = watcher(cert_issuer_clone, ListParams::default());
        let _ = watcher
            .try_for_each(|event| async {
                match event {
                    watcher::Event::Deleted(cert_issuer) => {
                        handle_delete(cert_issuer, context_clone.clone()).await;
                    }
                    _event => (),
                }
                Ok(())
            })
            .await;
    });

    let _ = futures::join!(controller_task, watcher_task);

    Ok(())
}

async fn reconcile(cert_issuer: CertIssuer, state: State) -> Result<ReconcilerAction, Error> {
    info!("Cert Issuer Reconciled: {:#?}", cert_issuer.meta().name);

    add_cert_issuer(&state, cert_issuer).await;

    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(60 * 10)),
    })
}

async fn handle_delete(cert_issuer: CertIssuer, state: State) {
    info!("Cert Issuer deleted: {:?}", cert_issuer);
    delete_cert_issuer(&state, cert_issuer).await;
}

fn error_policy(error: &Error, _state: State) -> ReconcilerAction {
    warn!("Error policy triggered: {:#?}", error);

    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(1)),
    }
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
