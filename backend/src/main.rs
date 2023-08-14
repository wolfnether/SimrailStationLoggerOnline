use core::time::Duration;
use std::time::SystemTime;

use common::db_data_struct::Log;
use common::simrail_data_struct::{Server, ServerResponse, Station, StationResponse};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::NotFoundError;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::{Data, Html, Json, Path};
use poem::{get, EndpointExt, Route};
use sqlx::migrate::MigrateDatabase;
use sqlx::types::chrono::{self, Utc};
use sqlx::{Executor, Sqlite, SqlitePool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = get_db().await?;

    let app = Route::new()
        .nest(
            "/dist",
            StaticFilesEndpoint::new("../frontend/dist/").index_file("index.html"),
        )
        .nest("/api", api_route())
        .at("/", get(index))
        .catch_error(not_found)
        .with(AddData::new(db.clone()));

    let polling_task = api_polling(db.clone());

    let serve_task = poem::Server::new(TcpListener::bind("0.0.0.0:8080")).run(app);
    tokio::select! {
        ret = polling_task => println!("polling task ended with {ret:?}"),
        ret = serve_task => println!("server ended with {ret:?}")
    };
    Ok(())
}

async fn get_db() -> anyhow::Result<SqlitePool> {
    const DB_URL: &str = "sqlite://sqlite.db";
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        Sqlite::create_database(DB_URL).await?;
    }

    let db = SqlitePool::connect(DB_URL).await?;
    db.acquire()
        .await?
        .execute(
            "CREATE TABLE IF NOT EXISTS log (server TEXT, station TEXT, player TEXT, date DATETIME)",
        )
        .await?;
    Ok(db)
}

fn api_route() -> Route {
    Route::new()
        .at("/servers", get(list_servers))
        .at("/logs/:server", get(list_log))
}

#[poem::handler]
fn index() -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

#[poem::handler]
async fn list_servers(db: Data<&SqlitePool>) -> anyhow::Result<Json<Vec<String>>> {
    let servers = sqlx::query_as("select * from log group by server")
        .fetch_all(&db.to_owned())
        .await?
        .iter()
        .map(|s: &Log| s.server.clone())
        .collect();
    Ok(Json(servers))
}

#[poem::handler]
async fn list_log(
    db: Data<&SqlitePool>,
    Path(server): Path<String>,
) -> anyhow::Result<Json<Vec<Log>>> {
    let logs = sqlx::query_as("select * from log where server = $1 order by station, date")
        .bind(server)
        .fetch_all(&db.to_owned())
        .await?;
    Ok(Json(logs))
}

async fn not_found(_: NotFoundError) -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

async fn api_polling(db: SqlitePool) -> Result<(), anyhow::Error> {
    loop {
        if let Ok(servers) = get_servers().await {
            for server in servers.iter().filter(|s: &&Server| s.is_active) {
                if let Ok(stations) = get_stations(&server.server_code).await {
                    for station in stations {
                        let last = sqlx::query_as(
                            "select * from log where server = $1 and station = $2 order by date",
                        )
                        .bind(&server.server_code)
                        .bind(&station.prefix)
                        .fetch_all(&db)
                        .await?
                        .iter()
                        .last()
                        .map(|l: &Log| l.player.clone())
                        .unwrap_or("BOT".into());

                        let actual = station
                            .dispatched_by
                            .first()
                            .map(|d| d.steam_id.clone())
                            .unwrap_or("BOT".into());

                        if last != actual {
                            sqlx::query(
                                "insert into log (server, station, player, date) VALUES ($1, $2, $3, datetime())",
                            )
                            .bind(&server.server_code)
                            .bind(&station.prefix)
                            .bind(&actual)
                            .execute(&db)
                            .await?;
                        }
                    }
                }
            }
        }
        sqlx::query("delete from log where date <= $1")
            .bind(chrono::DateTime::<Utc>::from(
                SystemTime::now() - Duration::from_secs(3600 * 24 * 3),
            ))
            .execute(&db)
            .await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn get_servers() -> Result<Vec<Server>, anyhow::Error> {
    Ok(reqwest::get("https://panel.simrail.eu:8084/servers-open")
        .await?
        .json::<ServerResponse>()
        .await?
        .data)
}

async fn get_stations(selected_server: &str) -> Result<Vec<Station>, anyhow::Error> {
    Ok(reqwest::get(format!(
        "https://panel.simrail.eu:8084/stations-open?serverCode={}",
        selected_server
    ))
    .await?
    .json::<StationResponse>()
    .await?
    .data)
}
