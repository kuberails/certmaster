use kube::{
    api::{Api, CustomResource, ListParams, Meta, Resource, WatchEvent},
    runtime::Informer,
    Client,
};
use kube_derive::CustomResource;
use rweb::openapi::Entity;
use rweb::Schema;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug, Default, Schema)]
struct Person {
    name: String,
}

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

fn main() -> () {
    env_logger::init();
    let crd = CertIssuer::crd();
    let schema = <CertIssuerSpec as Entity>::describe();

    println!("CRD: \n{}\n", serde_yaml::to_string(&crd)?);
    println!("SCHEMA: \n{}\n", serde_yaml::to_string(&schema)?);
}
