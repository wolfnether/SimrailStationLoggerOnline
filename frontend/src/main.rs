use common::db_data_struct::Log;
use itertools::Itertools;
use seed::{prelude::*, *};

struct Model {
    servers: Vec<String>,
    selected_server: String,
    logs: Vec<Log>,
    dark: bool,
    filter: String,
}

enum Msg {
    LoadLog,
    LoadServer,
    ServerLoaded(Vec<String>),
    LogLoaded(Vec<Log>),
    ServerChanged(String),
    StationChanged(String),
    ToggleDark,
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start("app", init, update, view);
}

fn view(model: &Model) -> Node<Msg> {
    div![
        style!(St::BackgroundColor => if model.dark {"#2b2b2b"} else {"#dfdfdf"},St::Color => if model.dark {"#dfdfdf"} else {"black"}),
        select![
            model
                .servers
                .iter()
                .map(|s| option![attrs!(At::Value=> s), s]),
            input_ev(Ev::Input, Msg::ServerChanged)
        ],
        button![
            if model.dark { "☀" } else { "☽" },
            input_ev(Ev::Click, |_| Msg::ToggleDark),
        ],
        {
            let stations = model
                .logs
                .iter()
                .group_by(|l| &l.station)
                .into_iter()
                .map(|(k, g)| (k, g.collect_vec()))
                .collect_vec();
            div![            select![
                option![attrs!(At::Value=> ""), ""],
                stations.iter().map(|(station, _)| option![attrs!(At::Value=> station), station]),
                input_ev(Ev::Input, Msg::StationChanged)
            ],
            stations.iter().filter(|(station,_)| model.filter.is_empty() || &&model.filter == station).enumerate().map(|(i,(station, log))| div![
            style!(St::BackgroundColor => if i % 2 == 0 {if model.dark {"#2b2b2b"} else {"#dfdfdf"}} else {if model.dark {"#3b3b3b"} else {"#cfcfcf"}}),
            p!(station),
            log.iter().map(|l| p!(
                style!(),
                {
                    let tz = - js_sys::Date::new_0().get_timezone_offset() as i64;
                    let date = l.date.clone() + time::Duration::minutes(tz );
                    format!("{}", date.format("%d/%m/%Y %H:%M:%S"))
                },
                " ",
                if l.player == "BOT" {a!("BOT")} else{
                    a!(attrs!(At::Href=> format!("https://steamcommunity.com/profiles/{}",l.player.clone()), At::Target => "_blank"),l.player.clone())
                }
            ))
        ])]
        }
    ]
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::LoadLog => {
            let selected_server = model.selected_server.clone();
            orders.perform_cmd(
                async move { Msg::LogLoaded(get_log(&selected_server).await.unwrap()) },
            );
        }
        Msg::LoadServer => {
            orders.perform_cmd(async { Msg::ServerLoaded(get_servers().await.unwrap()) });
        }
        Msg::ServerLoaded(servers) => {
            model.servers = servers;
            orders.send_msg(Msg::ServerChanged(
                model.servers.first().unwrap_or(&String::new()).clone(),
            ));
        }
        Msg::LogLoaded(logs) => model.logs = logs,
        Msg::ServerChanged(server) => {
            model.selected_server = server;
            orders.send_msg(Msg::LoadLog);
        }
        Msg::StationChanged(station) => model.filter = station,
        Msg::ToggleDark => model.dark = !model.dark,
    }
}

async fn get_servers() -> Result<Vec<String>, anyhow::Error> {
    let baseurl = web_sys::window().unwrap().origin();
    Ok(reqwest::get(format!("{baseurl}/api/servers"))
        .await?
        .json()
        .await?)
}

async fn get_log(server: &str) -> Result<Vec<Log>, anyhow::Error> {
    if server.is_empty() {
        return Ok(vec![]);
    }
    let baseurl = web_sys::window().unwrap().origin();
    Ok(reqwest::get(format!("{baseurl}/api/logs/{server}"))
        .await?
        .json()
        .await?)
}

fn init(_: Url, order: &mut impl Orders<Msg>) -> Model {
    order.stream(streams::interval(10000, || Msg::LoadLog));
    order.send_msg(Msg::LoadServer);
    Model {
        servers: vec![],
        logs: vec![],
        selected_server: String::new(),
        dark: true,
        filter: String::new(),
    }
}
