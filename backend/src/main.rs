use core::time::Duration;

use common::simrail_data_struct::{Server, ServerResponse, Station, StationResponse};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::NotFoundError;
use poem::listener::TcpListener;
use poem::web::{Html, Json};
use poem::{get, EndpointExt, Route};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Route::new()
        .nest(
            "/dist",
            StaticFilesEndpoint::new("../frontend/dist/").index_file("index.html"),
        )
        .nest("/api", api_route())
        .at("/", get(index))
        .catch_error(not_found);
    let polling_task = api_polling();

    let serve_task = poem::Server::new(TcpListener::bind("0.0.0.0:8080")).run(app);
    tokio::select! {
        ret = polling_task => println!("polling task ended with {ret:?}"),
        ret = serve_task => println!("server ended with {ret:?}")
    };
    Ok(())
}

fn api_route() -> Route {
    Route::new()
        .at("/servers", get(list_servers))
        .at("/station", get(list_station))
}

#[poem::handler]
fn index() -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

#[poem::handler]
fn list_servers() -> anyhow::Result<Json<Vec<()>>> {
    todo!()
}

#[poem::handler]
fn list_station() -> anyhow::Result<Json<Vec<()>>> {
    todo!()
}

async fn not_found(_: NotFoundError) -> Html<&'static str> {
    Html(include_str!("../../frontend/dist/index.html"))
}

async fn api_polling() {
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
