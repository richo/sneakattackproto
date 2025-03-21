use sneakattackproto::spreadsheet;
use sneakattackproto::structures::{self, UidMap};


fn main() -> Result<(), reqwest::Error> {
    // TODO(richo) Argparsing eventually
    let main_driver = 107;
    let benchmark_drivers = [1, 135, 965, 210];

    let mut uids = UidMap::new();
    let uids_list: Vec<structures::Uid> = spreadsheet::load_sneakattack_json("uidsSmall.json").unwrap();
    for uid in uids_list {
        uids.insert(uid.uid, uid);
    }
    let rallies: Vec<structures::Rally> = spreadsheet::load_sneakattack_json("2024rallies.json").expect("oh no");

    let slug = "ojibwe_forests_rally_2024";

    let active = rallies.iter().filter(|i| i.slug == slug).next().unwrap();

    let data = spreadsheet::build_data(active, main_driver, &benchmark_drivers);
    let mut book = spreadsheet::build_spreadsheet(active, &uids, main_driver, &benchmark_drivers).unwrap();
    book.save("timecomp.xlsx");

    Ok(())
}
