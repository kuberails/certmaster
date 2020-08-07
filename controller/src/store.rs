use crate::cert_issuer::CertIssuer;
use crate::certificate::Certificate;
use kube::api::Meta;
use kube::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Store(Arc<InnerStore>);

impl Clone for Store {
    fn clone(&self) -> Self {
        Store(Arc::clone(&self.0))
    }
}

pub struct InnerStore {
    readonly: ReadOnly,
    state: RwLock<State>,
}

struct ReadOnly {
    namespace: String,
    client: Client,
}

struct State {
    cert_issuers: Vec<CertIssuer>,
    certs: Vec<Certificate>,
}

impl Store {
    pub fn new(client: Client) -> Store {
        let state = State {
            cert_issuers: vec![],
            certs: vec![],
        };

        let namespace = std::env::var("NAMESPACE").unwrap_or("kuberails".to_string());

        let inner = InnerStore {
            readonly: ReadOnly { client, namespace },
            state: RwLock::new(state),
        };

        Store(Arc::new(inner))
    }

    pub fn get_ref(&self) -> &Arc<InnerStore> {
        &self.0
    }

    pub fn get_client(&self) -> &Client {
        &self.get_read_only().client
    }

    pub fn get_namespace(&self) -> &String {
        &self.get_read_only().namespace
    }

    fn get_read_only(&self) -> &ReadOnly {
        &self.get_ref().readonly
    }

    async fn add_cert_issuer(&self, cert_issuer: &CertIssuer) -> () {
        &self
            .get_ref()
            .state
            .write()
            .await
            .cert_issuers
            .push(cert_issuer.clone());
    }

    async fn delete_cert_issuer(&self, cert_issuer: &CertIssuer) -> () {
        &self
            .get_ref()
            .state
            .write()
            .await
            .cert_issuers
            .retain(|ci| ci.meta().name != cert_issuer.meta().name);
    }
}
