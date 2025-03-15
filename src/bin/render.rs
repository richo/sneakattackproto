use sneakattackproto::spreadsheet;
use sneakattackproto::structures;


fn main() -> Result<(), reqwest::Error> {
    // TODO(richo) Argparsing eventually
    let main_driver = 107;
    let benchmark_drivers = [1, 135];

    // let uids: Vec<structures::Uid> = fetch_sneakattack_json("uidsSmall.json")?;
    let uids: Vec<structures::Uid> = spreadsheet::load_sneakattack_json("uidsSmall.json").unwrap();
    let rallies: Vec<structures::Rally> = spreadsheet::load_sneakattack_json("2024rallies.json").expect("oh no");

    let slug = "ojibwe_forests_rally_2024";

    let active = rallies.iter().filter(|i| i.slug == slug).next().unwrap();

    let data = spreadsheet::build_data(active, main_driver, &benchmark_drivers);
    spreadsheet::build_spreadsheet(active, main_driver, &benchmark_drivers);

    Ok(())
}
