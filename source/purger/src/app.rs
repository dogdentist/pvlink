use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{config, errorln, outputln};

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

async fn purge() -> anyhow::Result<()> {
    let timestamp = chrono::Utc::now();
    let db_conn = open_db_connection().await?;

    sqlx::query(
        r#"
            DELETE FROM tb_link
            WHERE tb_link_expires IS NOT NULL AND tb_link_expires < $1
        "#,
    )
    .bind(timestamp)
    .execute(&db_conn)
    .await?;

    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    let config = config::object();

    loop {
        if let Err(e) = purge().await {
            errorln!("puring job failed, error: {}", e.to_string());
        } else {
            outputln!("purge job completed");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            config.purge_interval_minutes * 60,
        ))
        .await;
    }
}
