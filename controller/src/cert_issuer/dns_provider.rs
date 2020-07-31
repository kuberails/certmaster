use cloudflare::Cloudflare;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "provider")]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    DigitalOcean(DigitalOcean),
    Cloudflare(Cloudflare),
    Route53(Route53),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DigitalOcean {
    auth_token: String,
}

mod cloudflare {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    #[serde(untagged)]
    pub enum Cloudflare {
        EmailAndKey(EmailAndKey),
        ApiToken(ApiToken),
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct EmailAndKey {
        api_email: String,
        api_key: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ApiToken {
        #[serde(rename = "apiToken")]
        token: String,
    }
}
