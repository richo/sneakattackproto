use reqwest;
use serde;
use serde_json;
use std::fs;
use rust_xlsxwriter::{self as xls, Workbook, XlsxError};

use crate::structures;

const SNEAK_ATTACK_BASE: &'static str = "https://sneakattackrally.com/ARACombinerThing/data";
fn fetch_sneakattack_json<'a, T: serde::de::DeserializeOwned>(name: &'a str) -> Result<T, reqwest::Error> {
    let path = format!("{}/{}", SNEAK_ATTACK_BASE, name);
    let body = reqwest::blocking::get(path)?
        .json();
    body
}

pub fn load_sneakattack_json<'a, T: serde::de::DeserializeOwned>(name: &'a str) -> Result<T, ()> {
    let fh = fs::File::open(name).unwrap();
    let res = serde_json::from_reader(fh);
    Ok(res.unwrap())
}

fn get_rallies_by_year(year: usize) -> Result<(), ()> {
    Ok(())
}

pub struct RallyData {
}

pub fn build_data(rally: &structures::Rally, driver: usize, benchmarks: &[usize]) -> RallyData {
    let driver_entry = rally.entry_by_driver_number(driver).unwrap();
    let driver_class = driver_entry.class;

    RallyData {}
}

pub fn build_spreadsheet(rally: &structures::Rally, driver: usize, benchmarks: &[usize]) -> Result<(), XlsxError> {
    let bold_format = xls::Format::new().set_bold();
    let stage_name_format = xls::Format::new()
        .set_border(xls::FormatBorder::Thin);
    let stage_length_format = xls::Format::new()
        .set_border(xls::FormatBorder::Thin)
        .set_num_format("0.00");
    let heading_format = xls::Format::new()
        .set_border(xls::FormatBorder::Medium)
        .set_bold();

    let stage_time_format = xls::Format::new();
    let invalid_time_format = xls::Format::new()
        .set_background_color(xls::Color::Theme(2,3));
    let class_win_format = xls::Format::new()
        .set_background_color(xls::Color::Theme(6,3));
    let overall_class_win_format = xls::Format::new()
        .set_background_color(xls::Color::Theme(9, 3));

    let delta_invalid_format = xls::Format::new();
    let delta_format = xls::Format::new();
    let delta_faster_format = xls::Format::new()
        .set_background_color(xls::Color::Theme(6,2));

    let driver = rally.entries.iter().filter(|x| x.number == driver).next().unwrap();
    let benchmarks: Vec<_> = rally.entries.iter().filter(|x| benchmarks.contains(&x.number)).collect();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Welp time to do a bunch of bookkeeping!
    let stage_start_row = 2;
    // Title/Stage names columns
    worksheet.set_column_width(0, 18)?;
    worksheet.write_with_format(0, 0, &rally.title, &bold_format)?;
    worksheet.write_with_format(1, 0, "Stage Name", &heading_format)?;
    worksheet.write_with_format(1, 1, "Length", &heading_format)?;

    // Milage column
    worksheet.set_column_width(1, 8)?;

    let format_time = |time: &structures::StageTime, overall_win: &Option<structures::StageTime>, category_class_win: &Option<structures::StageTime>| {
        if !time.is_valid() {
            return &invalid_time_format;
        } else if Some(time) == overall_win.as_ref() {
            return &overall_class_win_format;
        } else if Some(time) == category_class_win.as_ref() {
            return &class_win_format;
        } else {
            return &stage_time_format;
        }
    };

    let format_delta = |delta: structures::Delta| {
        match delta.kind {
            structures::DeltaKind::Invalid |
                structures::DeltaKind::Equal => &delta_invalid_format,
            structures::DeltaKind::Faster => &delta_faster_format,
            structures::DeltaKind::Slower => &delta_format
        }
    };

    let driver_column = 2;
    let benchmark_start_column = 4;

    worksheet.write_with_format(1, driver_column,
        format!("{}", driver.number), // TODO(richo) Do the uid lookup thing to figure out who we are
        &heading_format)?;
    for (i, benchmark) in benchmarks.iter().enumerate() {
        worksheet.write_with_format(1, benchmark_start_column + (i * 2) as u16,
            format!("{}", benchmark.number),
            &heading_format)?;
        worksheet.write_with_format(1, benchmark_start_column + 1 + (i * 2) as u16,
            "Diff s/mi",
            &heading_format)?;
    }

    for (stage_number, stage) in rally.stages.iter().enumerate() {
        worksheet.write_with_format(stage_start_row + stage_number as u32, 0, &stage.name, &stage_name_format)?;
        worksheet.write_with_format(stage_start_row + stage_number as u32, 1, stage.length, &stage_length_format)?;

        let overall_win = rally.entries.iter()
            .filter(|x| x.class == driver.class)
            .map(|x| x.times[stage_number])
            .filter(|x| x.is_valid())
            .min();

        let class_win = rally.entries.iter()
            .filter(|x| x.class == driver.class)
            .filter(|x| x.category == driver.category)
            .map(|x| x.times[stage_number])
            .filter(|x| x.is_valid())
            .min();

        let driver_time = driver.times[stage_number];

        worksheet.write_with_format(stage_start_row + stage_number as u32, driver_column, driver_time.to_string(), format_time(&driver_time, &overall_win, &class_win))?;

        for (i, benchmark) in benchmarks.iter().enumerate() {
            let benchmark_time = benchmark.times[stage_number];

            worksheet.write_with_format(stage_start_row + stage_number as u32,
                benchmark_start_column + (i * 2) as u16,
                benchmark_time.to_string(),
                format_time(&benchmark_time, &overall_win, &class_win))?;
            if benchmark_time.is_valid() && driver_time.is_valid() {
                let delta = driver_time.diff_per_mile(&benchmark_time, stage.length);
                worksheet.write_with_format(stage_start_row + stage_number as u32,
                    benchmark_start_column + 1 + (i * 2) as u16,
                    delta.to_string(),
                    format_delta(delta))?;
            }
        }
    }

    workbook.save("hello.xlsx")?;

    Ok(())
}
