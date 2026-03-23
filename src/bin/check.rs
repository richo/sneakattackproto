use tokio;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use sneakattackproto::spreadsheet;
use sneakattackproto::structures::{self, UidMap};
use std::sync::OnceLock;
use regex::Regex;

// It'd be better if this was Cow or whatever
#[derive(Clone)]
struct RallyState {
    uids: HashMap<usize, structures::Uid>,
    rallies: HashMap<usize, HashMap<String, structures::Rally>>,
}

// Keep these in the order you want them displayed in the web interface
const RALLY_DATA: &[(usize, &'static str)] = &[
    (2026, "2026rallies.json"),
    (2025, "2025rallies.json"),
    (2024, "2024rallies.json"),
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
        rallies.insert(*year, data);
    }

    RallyState {
        uids,
        rallies,
    }
}

fn main() {
    let state = build_state();

    println!("state builds")
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TimeComp {
    driver: usize,
    benchmarks: Vec<usize>,
    event: String,
}
