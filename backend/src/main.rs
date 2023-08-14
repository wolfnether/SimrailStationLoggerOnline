use core::time::Duration;

use common::simrail_data_struct::{Server, ServerResponse, Station, StationResponse};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::NotFoundError;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::{Data, Html, Json};
use poem::{get, EndpointExt, Route};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Acquire, Executor, Sqlite, SqlitePool};

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
        .at("/station", get(list_station))
        .at("/log", get(list_log))
}

#[poem::handler]
fn index() -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

#[poem::handler]
fn list_servers(db: Data<&SqlitePool>) -> anyhow::Result<Json<Vec<String>>> {
    todo!()
}

#[poem::handler]
fn list_station(db: Data<&SqlitePool>) -> anyhow::Result<Json<Vec<String>>> {
    todo!()
}
#[poem::handler]
fn list_log(db: Data<&SqlitePool>) -> anyhow::Result<Json<Vec<String>>> {
    todo!()
}

async fn not_found(_: NotFoundError) -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

async fn api_polling(db: SqlitePool) {
    loop {
        if let Ok(servers) = get_servers().await {
            for server in servers.iter().filter(|s| s.is_active) {
                if let Ok(stations) = get_stations(&server.server_code).await {
                    for station in stations {
                        println!("{station:?}")
                    }
                }
            }
        }
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
