use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use http_body_util::{Empty, combinators::BoxBody};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    header,
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use redis::AsyncTypedCommands;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{
    accessln,
    config::{self},
    constants, errorln, outputln,
};

async fn push_to_analytic(ip: Option<String>, short_link: String) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct BrokerMessage {
        #[serde(rename = "i")]
        ip: Option<String>,
        #[serde(rename = "l")]
        short_link: String,
    }

    let config = config::object();
    let object = BrokerMessage { ip, short_link };
    let queue_conn = nats::asynk::connect(&config.analytic_queue_fqdn).await?;

    queue_conn
        .publish(
            constants::ANALYTIC_QUEUE_NAME,
            serde_json::to_string(&object)?,
        )
        .await?;

    Ok(())
}

async fn open_db_connection() -> anyhow::Result<Pool<Postgres>> {
    let config = config::object();

    let connection_string = format!(
        "postgres://{}:{}@{}/{}",
        config.pg_username, config.pg_password, config.pg_fqdn, config.pg_dbname
    );

    Ok(PgPoolOptions::new()
        .max_connections(config.pg_max_conns)
        .connect(&connection_string)
        .await?)
}

async fn fetch_link(short_link: &str) -> anyhow::Result<Option<String>> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct CacheValue {
        #[serde(rename = "s")]
        short_link: String,
        #[serde(rename = "l")]
        long_link: String,
    }

    let config = config::object();
    let client = redis::Client::open("redis://".to_owned() + &config.cache_fqdn)?;
    let mut cache_conn = client.get_multiplexed_tokio_connection().await?;
    let cache_value = cache_conn.get(short_link).await?;

    // cache hit
    if let Some(cache_value) = cache_value {
        if let Ok(cache_value) = serde_json::from_str::<CacheValue>(&cache_value) {
            return Ok(Some(cache_value.long_link));
        }
    };

    // cache miss
    let db_conn = open_db_connection().await?;

    let row: Option<(String,)> = sqlx::query_as(
        r#"
            SELECT tb_link_long FROM tb_link
            WHERE tb_link_short = $1
                AND (tb_link_expires IS NULL OR tb_link_expires < NOW())
            LIMIT 1;
        "#,
    )
    .bind(short_link)
    .fetch_optional(&db_conn)
    .await?;

    if let Some((long_link,)) = row {
        let value = CacheValue {
            short_link: short_link.to_string(),
            long_link: long_link.clone(),
        };

        cache_conn
            .set(short_link, serde_json::to_string(&value)?)
            .await?;

        return Ok(Some(long_link));
    }

    Ok(None)
}

fn create_redirect_response(
    target_uri: String,
) -> anyhow::Result<Response<BoxBody<Bytes, Infallible>>> {
    let config = config::object();

    Ok(Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header(header::LOCATION, target_uri)
        .header(header::SERVER, config.server_name.to_owned())
        .body(BoxBody::new(Empty::<Bytes>::new()))?)
}

async fn request_handler(
    address: Arc<Option<String>>,
    path: &str,
) -> anyhow::Result<Response<BoxBody<Bytes, Infallible>>> {
    let config = config::object();

    if path == "/" {
        if !config.fallback_url.is_empty() {
            create_redirect_response(config.fallback_url.clone())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::SERVER, config.server_name.to_owned())
                .body(BoxBody::new(Empty::<Bytes>::new()))?)
        }
    } else {
        let short_link = if let Some(path) = path.strip_prefix('/') {
            path
        } else {
            path
        };

        if let Some(long_link) = fetch_link(short_link).await? {
            let ip = (*address).to_owned();
            let short_link = short_link.to_owned();

            tokio::spawn(async move {
                if let Err(e) = push_to_analytic(ip, short_link).await {
                    errorln!("failed to push to analytics, error: {}", e.to_string());
                }
            });

            Ok(Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(header::LOCATION, long_link)
                .header(header::SERVER, config.server_name.to_owned())
                .body(BoxBody::new(Empty::<Bytes>::new()))?)
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::SERVER, config.server_name.to_owned())
                .body(BoxBody::new(Empty::<Bytes>::new()))?)
        }
    }
}

pub async fn server() -> anyhow::Result<()> {
    let bind_address = constants::LISTEN_ADDRESS;

    outputln!("creating a server on {bind_address}");

    let listener = tokio::net::TcpListener::bind(bind_address).await?;

    let service_function = async move |
                                       req: Request<Incoming>|
                -> anyhow::Result<Response<BoxBody<Bytes, Infallible>>> {
        let config = config::object();
        let address = if let Some(forwarded_for) = req.headers().get("x-real-ip") {
            let address = String::from_utf8_lossy(forwarded_for.as_bytes()).to_string();

            if let Ok(address) = address.parse::<IpAddr>() {
                Arc::new(Some(address.to_string()))
            } else if let Ok(address) = address.parse::<SocketAddr>() {
                Arc::new(Some(address.ip().to_string()))
            } else {
                Arc::new(None)
            }
        } else {
            Arc::new(None)
        };

        let click_ip = &address.as_deref().unwrap_or("N/A");
        let path = req.uri().path();

        if req.method() == Method::GET {
            match request_handler(address.clone(), path).await {
                Ok(v) => {
                    let status = v.status().as_u16();

                    if let Some(location) = v.headers().get("location") {
                        let location = String::from_utf8_lossy(location.as_bytes()).to_string();

                        accessln!(
                            click_ip,
                            "status: {status}; path: {path}; redirect: {location}"
                        );
                    } else {
                        accessln!(
                            click_ip,
                            "status: {status}; path: {path}"
                        );
                    };

                    Ok(v)
                },
                Err(e) => {
                    accessln!(
                        click_ip,
                        "status: 500 path: {path}; internal error, {}",
                        e.to_string()
                    );

                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header(header::SERVER, config.server_name.to_owned())
                        .body(BoxBody::new(Empty::<Bytes>::new()))?)
                }
            }
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::SERVER, config.server_name.to_owned())
                .body(BoxBody::new(Empty::<Bytes>::new()))?)
        }
    };

    loop {
        let (stream, address) = match listener.accept().await {
            Ok(v) => (v.0, v.1.to_string()),
            Err(e) => {
                accessln!(
                    "N/A",
                    "failed to accept a connection in the server, {}",
                    e.to_string()
                );

                continue;
            }
        };

        let address = Arc::new(address);
        let io = TokioIo::new(stream);
        let service = { service_fn(move |req: Request<Incoming>| service_function(req)) };

        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                accessln!(&address, "failed to handle connection, {}", e.to_string());
            }
        });
    }
}
