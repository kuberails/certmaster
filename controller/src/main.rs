use certmaster::cert_issuer::CertIssuer;
use certmaster::certificate::{self, Certificate};
use certmaster::error::Error;
use certmaster::store::Store;
use futures::prelude::*;
use kube::{
    api::{Api, ListParams},
    Client,
};
use kube_runtime::watcher;
use tokio;
use tokio::task;
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;

    let cert_issuer: Api<CertIssuer> = Api::all(client.clone());
    let store = Store::new(client.clone());

    let certs: Api<Certificate> = Api::all(client.clone());

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

async fn cert_watcher(api: Api<Certificate>, store: Store) -> Result<(), JoinError> {
    task::spawn(async move {
        let lp = ListParams::default()
            .timeout(60)
            .labels("app.kubernetes.io/managed-by=certmaster.kuberails.com/v1,certmaster.kuberails.com/certIssuer");

        let watcher = watcher(api, lp);

        let _ = watcher
            .try_for_each(|event| handle_cert_events(event, store.clone()))
            .await;
    })
    .await
}

async fn handle_cert_issuer_events(
    event: watcher::Event<CertIssuer>,
    store: Store,
) -> Result<(), watcher::Error> {
    match event {
        watcher::Event::Applied(cert_issuer) => {}
        _ => (),
    }

    Ok(())
}

async fn handle_cert_events(
    event: watcher::Event<Certificate>,
    store: Store,
) -> Result<(), watcher::Error> {
    println!("CERT: {:#?}", event);

    Ok(())
}
