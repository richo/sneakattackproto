use reqwest;
use serde;
use serde_json;
use std::fs;
use rust_xlsxwriter::{Workbook, XlsxError};

mod structures;

const SNEAK_ATTACK_BASE: &'static str = "https://sneakattackrally.com/ARACombinerThing/data";
fn fetch_sneakattack_json<'a, T: serde::de::DeserializeOwned>(name: &'a str) -> Result<T, reqwest::Error> {
    let path = format!("{}/{}", SNEAK_ATTACK_BASE, name);
    let body = reqwest::blocking::get(path)?
        .json();
    body
}

fn load_sneakattack_json<'a, T: serde::de::DeserializeOwned>(name: &'a str) -> Result<T, ()> {
    let fh = fs::File::open(name).unwrap();
    let res = serde_json::from_reader(fh);
    Ok(res.unwrap())
}

fn get_rallies_by_year(year: usize) -> Result<(), ()> {
    Ok(())
}

struct RallyData {
}

fn build_data(rally: &structures::Rally, driver: usize, benchmarks: &[usize]) -> RallyData {
    let driver_entry = rally.entry_by_driver_number(driver).unwrap();
    let driver_class = driver_entry.class;

    RallyData {}
}

fn lower_to_spreadsheet(data: RallyData) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write(0, 0, "Hello")?;
    workbook.save("hello.xlsx")?;

    Ok(())
}

fn main() -> Result<(), reqwest::Error> {
    // TODO(richo) Argparsing eventually
    let main_driver = 107;
    let benchmark_drivers = [1, 135];

    // let uids: Vec<structures::Uid> = fetch_sneakattack_json("uidsSmall.json")?;
    let uids: Vec<structures::Uid> = load_sneakattack_json("uidsSmall.json").unwrap();
    let rallies: Vec<structures::Rally> = load_sneakattack_json("2024rallies.json").expect("oh no");

    let slug = "ojibwe_forests_rally_2024";

    let active = rallies.iter().filter(|i| i.slug == slug).next().unwrap();

    let data = build_data(active, main_driver, &benchmark_drivers);
    lower_to_spreadsheet(data);

    Ok(())
}
