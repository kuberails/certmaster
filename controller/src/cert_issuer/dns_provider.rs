use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "provider")]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    DigitalOcean(BasicAuth),
    Cloudflare(BasicAuth),
    Route53(Route53),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuth {
    key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Route53 {
    access_key: String,
    secret_access_key: String,
    region: String,
    profile: Option<String>,
    hosted_zone_id: Option<String>,
}
