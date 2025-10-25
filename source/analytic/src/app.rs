use crate::{config, constants, errorln, geo, outputln};
use anyhow::anyhow;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, prelude::FromRow};

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

async fn increment_link(short_link: &str, country: &String) -> anyhow::Result<()> {
    #[derive(FromRow)]
    struct LinkGetId {
        #[sqlx(rename = "tb_link_id")]
        id: i64,
    }

    let db_conn = open_db_connection().await?;
    let mut db_txn = db_conn.begin().await?;

    let link_row: Option<LinkGetId> = sqlx::query_as(
        r#"
            SELECT tb_link_id
            FROM tb_link
            WHERE tb_link_short = $1
        "#,
    )
    .bind(short_link)
    .fetch_optional(&mut *db_txn)
    .await?;

    if let Some(link_row) = link_row {
        sqlx::query(
            r#"
            UPDATE tb_link
            SET tb_link_clicks = tb_link_clicks + 1
            WHERE tb_link_id = $1
        "#,
        )
        .bind(link_row.id)
        .execute(&mut *db_txn)
        .await?;

        sqlx::query(
            r#"
                INSERT INTO tb_link_clicks (tb_link_clicks_link_id, tb_link_clicks_country, tb_link_clicks_clicks)
                VALUES ($1, $2, 1)
                ON CONFLICT (tb_link_clicks_link_id, tb_link_clicks_country)
                DO UPDATE SET tb_link_clicks_clicks = tb_link_clicks.tb_link_clicks_clicks + 1
            "#,
        )
        .bind(link_row.id)
        .bind(country)
        .execute(&mut *db_txn)
        .await?;

        db_txn.commit().await?;
    }

    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    #[derive(serde::Deserialize)]
    struct BrokerMessage {
        #[serde(rename = "i")]
        ip: Option<String>,
        #[serde(rename = "l")]
        short_link: String,
    }

    let config = config::object();

    let queue_conn = nats::asynk::connect(&config.analytic_queue_fqdn).await?;

    let queue = queue_conn
        .queue_subscribe(
            constants::ANALYTIC_QUEUE_NAME,
            constants::ANALYTIC_QUEUE_NAME,
        )
        .await?;

    while let Some(message) = queue.next().await {
        match serde_json::from_str::<BrokerMessage>(&String::from_utf8_lossy(&message.data)) {
            Ok(message) => {
                let click_ip = &message.ip.as_deref().unwrap_or("N/A");

                let geo_request = async || {
                    if let Some(ip) = &message.ip {
                        geo::request(ip).await
                    } else {
                        Ok(constants::COUNTRY_NULL.to_owned())
                    }
                };

                let country = match geo_request().await {
                    Ok(country) => country,
                    Err(e) => {
                        errorln!(
                            "failed to retrive the geolocation of the ip; ip: {}, short link: {}; error: {}",
                            click_ip,
                            message.short_link,
                            e.to_string()
                        );

                        constants::COUNTRY_NULL.to_owned()
                    }
                };

                if let Err(e) = increment_link(&message.short_link, &country).await {
                    errorln!(
                        "failed to increment a click of a link; ip: {}, short link: {}; error: {}",
                        click_ip,
                        message.short_link,
                        e.to_string()
                    )
                } else {
                    outputln!(
                        "incremented click, ip: {}; country: {}, short link: {}",
                        click_ip,
                        country,
                        message.short_link
                    )
                }
            }
            Err(e) => errorln!(
                "corrupted message from the broker, error: {}",
                e.to_string()
            ),
        }
    }

    Err(anyhow!("lost connection with the queue"))
}
