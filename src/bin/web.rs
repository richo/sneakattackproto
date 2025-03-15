use tokio;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use axum::{extract::Form, response::Html};
use axum_extra::extract::Query;

use std::collections::HashMap;

use sneakattackproto::spreadsheet;
use sneakattackproto::structures;

// It'd be better if this was Cow or whatever
#[derive(Clone)]
struct RallyState {
    uids: HashMap<usize, structures::Uid>,
    rallies: HashMap<usize, HashMap<String, structures::Rally>>,
}

// Keep these in the order you want them displayed in the web interface
const RALLY_DATA: &[(usize, &'static str)] = &[
    (2025, "2025rallies.json"),
    (2024, "2024rallies.json"),
];

fn build_state() -> RallyState {

    let mut uids = HashMap::<usize, structures::Uid>::new();
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
        rallies.insert(*year, data);
    }

    RallyState {
        uids,
        rallies,
    }
}

#[tokio::main]
async fn main() {
    let state = build_state();

    let app = Router::new()
        .route("/", get(show_form))
        .route("/render", get(render_timecomp))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post">
                    <label for="name">
                        Enter your name:
                        <input type="text" name="name">
                    </label>

                    <label>
                        Enter your email:
                        <input type="text" name="email">
                    </label>

                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TimeComp {
    driver: usize,
    benchmarks: Vec<usize>,
    event: String,
}

async fn render_timecomp(input: Query<TimeComp>) {
    dbg!(&input);
}
