use tokio;
use axum::{
    body::Body,
    routing::{get, post},
    http::{header, HeaderMap, StatusCode},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use axum::extract::{State, Form};
use axum::response::{Html, IntoResponse};
use axum_extra::extract::Query;
use tower_http::services::ServeFile;

use std::collections::HashMap;

use sneakattackproto::spreadsheet;
use sneakattackproto::structures::{self, UidMap};
use std::sync::OnceLock;
use regex::Regex;

// It'd be better if this was Cow or whatever
#[derive(Clone)]
struct RallyState {
    uids: HashMap<usize, structures::Uid>,
    rallies: HashMap<String, HashMap<String, structures::Rally>>,
}

// Keep these in the order you want them displayed in the web interface
const RALLY_DATA: &[(&'static str, &'static str)] = &[
    ("2025", "2025rallies.json"),
    ("2024", "2024rallies.json"),
    // ("nonARA", "nonARArallies.json"),
];

fn build_state() -> RallyState {

    let mut uids = UidMap::new();
    let uids_list: Vec<structures::Uid> = spreadsheet::load_sneakattack_json("uidsSmall.json").unwrap();
    for uid in uids_list {
        uids.insert(uid.uid, uid);
    }

    let mut rallies = HashMap::new();
    for (year, data_file) in RALLY_DATA {
        let mut data = HashMap::new();
        let rallies_list: Vec<structures::Rally> = spreadsheet::load_sneakattack_json(data_file).unwrap();
        for rally in rallies_list {
            data.insert(rally.slug.clone(), rally);
        }
        rallies.insert((*year).into(), data);
    }

    let non_ara: structures::NonARARallies = spreadsheet::load_sneakattack_json("nonARArallies.json").unwrap();
    let mut data = HashMap::new();
    for rally in non_ara.archive {
        data.insert(rally.slug.clone(), rally.into());
    }
    rallies.insert("nonARA".into(), data);

    RallyState {
        uids,
        rallies,
    }
}

#[tokio::main]
async fn main() {
    let state = build_state();

    let app = Router::new()
        .route_service("/", ServeFile::new("html/timecomp.html"))
        .route("/render", get(render_timecomp))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TimeComp {
    driver: usize,
    benchmarks: Vec<usize>,
    event: String,
}

async fn render_timecomp(input: Query<TimeComp>, State(state): State<RallyState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Split up year and slug
    static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| { Regex::new(r"^(\d+)\|(.+)$").unwrap() });

    let (_, [year, slug]) = re.captures_iter(&input.event)
        .map(|c| c.extract()).next()
        .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse year|slug"),
            ))?;

    let active = &state.rallies[year][slug];

    let mut book = spreadsheet::build_spreadsheet(&active, &state.uids, input.driver, &input.benchmarks).map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build spreadsheet: {e}"),
    ))?;
    let buf = book.save_to_buffer().unwrap();

    let content_disposition_header = format!("attachment; filename=\"{}_{}.xlsx\"", &input.event, &input.driver);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/toml; charset=utf-8".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, content_disposition_header.parse().unwrap());

    Ok((headers, buf))
}
