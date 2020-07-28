use certmaster::cert_issuer::CertIssuer;
use certmaster::certificate::{self, Certificate};
use certmaster::consts::labels::{CACHED, CERT_ISSUER, MANAGED_BY_KEY, MANAGED_BY_VALUE};
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
        let lp = ListParams::default().timeout(60).labels(&format!(
            "{managed_by_key}={managed_by_value},{cert_issuer},{cached}!=true",
            managed_by_key = MANAGED_BY_KEY,
            managed_by_value = MANAGED_BY_VALUE,
            cert_issuer = CERT_ISSUER,
            cached = CACHED
        ));

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
        watcher::Event::Applied(cert_issuer) => {
            let res = certificate::cache_and_create_for_namespaces(&store, &cert_issuer).await;

            if let Ok(certificates) = res {
                //TODO:
                // save certificates to store
            }
            ()
        }
        _ => (),
    }

    Ok(())
}

async fn handle_cert_events(
    event: watcher::Event<Certificate>,
    store: Store,
) -> Result<(), watcher::Error> {
    println!("CERT: {:?}", event);

    Ok(())
}
