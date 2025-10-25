use std::sync::OnceLock;

use anyhow::Context;

use crate::outputln;

#[derive(Debug)]
pub struct Object {
    pub log_days_retention: usize,
    pub pg_fqdn: String,
    pub pg_username: String,
    pub pg_password: String,
    pub pg_dbname: String,
    pub pg_max_conns: u32,
    pub cache_fqdn: String,
    pub analytic_queue_fqdn: String,
    pub analytic_ipinfo_token: String,
}

static OBJECT: OnceLock<Object> = OnceLock::new();

pub fn object<'a>() -> &'a Object {
    OBJECT.get().expect("config was not initialized")
}

pub fn load() -> anyhow::Result<()> {
    let object: Object = Object {
        log_days_retention: std::env::var("PVLINK_LOG_RETENTION")
            .context("missing 'PVLINK_LOG_RETENTION'")?
            .parse()
            .context("bad 'PVLINK_LOG_RETENTION' value")?,
        pg_fqdn: std::env::var("PVLINK_DB_FQDN").context("missing 'PVLINK_DB_FQDN'")?,
        pg_username: std::env::var("PVLINK_DB_USER").context("missing 'PVLINK_DB_USER'")?,
        pg_password: std::env::var("PVLINK_DB_PASSWORD").context("missing 'PVLINK_DB_PASSWORD'")?,
        pg_dbname: std::env::var("PVLINK_DB_NAME").context("missing 'PVLINK_DB_NAME'")?,
        pg_max_conns: std::env::var("PVLINK_DB_MAX_CONNS")
            .context("missing 'PVLINK_DB_MAX_CONNS'")?
            .parse()
            .context("bad 'PVLINK_DB_MAX_CONNS' value")?,
        cache_fqdn: std::env::var("PVLINK_CACHE_FQDN").context("missing 'PVLINK_CACHE_FQDN'")?,
        analytic_queue_fqdn: std::env::var("PVLINK_ANALYTIC_QUEUE_FQDN")
            .context("missing 'PVLINK_ANALYTIC_QUEUE_FQDN'")?,
        analytic_ipinfo_token: std::env::var("ANALYTIC_IPINFO_TOKEN")
            .context("missing 'ANALYTIC_IPINFO_TOKEN'")?,
    };

    OBJECT.set(object).expect("config was already initialized");

    outputln!("configuration was loaded");

    Ok(())
}
