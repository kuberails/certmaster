use certmaster::crd::{Cert, CertIssuer};
use certmaster::store::Store;
use futures::prelude::*;
use kube::{
    api::{Api, ListParams, Meta},
    Client,
};
use kube_runtime::watcher;
use log::{info, warn};
use tokio;
use tokio::task;
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;

    let cert_issuer: Api<CertIssuer> = Api::all(client.clone());
    let store = Store::new(client.clone());
    let certs: Api<Cert> = Api::all(client.clone());

    let _ = tokio::join!(
        cert_issuer_watcher(cert_issuer, store.clone()),
        cert_watcher(certs, store)
    );

    Ok(())
}

async fn cert_issuer_watcher(api: Api<CertIssuer>, store: Store) -> Result<(), JoinError> {
    task::spawn(async move {
        let watcher = watcher(api, ListParams::default());

        let _ = watcher
            .try_for_each(|event| handle_cert_issuer_events(event, store.clone()))
            .await;
    })
    .await
}

async fn cert_watcher(api: Api<Cert>, store: Store) -> Result<(), JoinError> {
    task::spawn(async move {
        let watcher = watcher(api, ListParams::default());

        let _ = watcher
            .try_for_each(|event| handle_cert_events(event, store.clone()))
            .await;
    })
    .await
}

async fn handle_cert_issuer_events(
    watcher: watcher::Event<CertIssuer>,
    store: Store,
) -> Result<(), watcher::Error> {
    Ok(())
}

async fn handle_cert_events(
    event: watcher::Event<Cert>,
    store: Store,
) -> Result<(), watcher::Error> {
    Ok(())
}
