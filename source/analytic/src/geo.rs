use anyhow::anyhow;
use redis::AsyncTypedCommands;

use crate::{config, constants};

pub async fn request(ip: &str) -> anyhow::Result<String> {
    let config = config::object();
    let client = redis::Client::open("redis://".to_owned() + &config.cache_fqdn)?;
    let mut cache_conn = client.get_multiplexed_tokio_connection().await?;
    let cache_value: Option<String> = cache_conn.get(ip).await?;

    // cache hit
    if let Some(country_code) = cache_value {
        return Ok(country_code);
    };

    // cache miss
    for _ in 0..3 {
        let res = reqwest::get(
            "https://api.ipinfo.io/lite/".to_owned()
                + ip
                + "/country?token="
                + &config.analytic_ipinfo_token,
        )
        .await?;

        match res.status() {
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                if let Some(retry_after) = res.headers().get("retry-after") {
                    let seconds: u64 = String::from_utf8_lossy(retry_after.as_bytes()).parse()?;

                    tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
                    continue;
                }

                return Err(anyhow!("got TOO_MANY_REQUESTS status but no retry after"));
            }
            reqwest::StatusCode::OK => {
                let country = res.text().await?.trim().to_owned();

                // save to the cache
                cache_conn.set(ip, country.clone()).await?;

                return Ok(country);
            }
            _ => return Ok(constants::COUNTRY_NULL.to_owned()),
        }
    }

    Err(anyhow!(
        "after 3 attempts we failed to retrieve the geo data from the ip"
    ))
}
