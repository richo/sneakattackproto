use reqwest;
use serde;
use serde_json;
use std::fs;
use std::fmt;
use std::error::Error;
use rust_xlsxwriter::{self as xls, Workbook, XlsxError};

use crate::structures::{self, UidMap};

mod format {
    use super::xls;
    pub(super) struct Formats {
        pub bold: xls::Format,
        pub stage_name: xls::Format,
        pub stage_length: xls::Format,
        pub stage_time: xls::Format,
        pub heading: xls::Format,
        pub invalid_time: xls::Format,
        pub overall_class_win: xls::Format,
        pub class_win: xls::Format,
        pub delta: xls::Format,
        pub delta_invalid: xls::Format,
        pub delta_faster: xls::Format,
        pub super_rally: xls::Format,
        pub driver_names: xls::Format,
    }

    pub(super) fn get_formats() -> Formats {
        let stage_time = xls::Format::new()
                .set_align(xls::FormatAlign::Right)
                .set_border_left(xls::FormatBorder::Thin);

        let delta = xls::Format::new()
            .set_border_right(xls::FormatBorder::Thin);

        Formats {
            bold: xls::Format::new()
                .set_bold(),
            stage_name: xls::Format::new()
                .set_border(xls::FormatBorder::Thin),
            stage_length: xls::Format::new()
                .set_border(xls::FormatBorder::Thin)
                .set_num_format("0.00"),
            heading: xls::Format::new()
                .set_border(xls::FormatBorder::Medium)
                .set_bold(),

            invalid_time: stage_time.clone()
                .set_background_color(xls::Color::Theme(2,3)),
            overall_class_win: stage_time.clone()
                .set_background_color(xls::Color::Theme(9, 3)),
            class_win: stage_time.clone()
                .set_background_color(xls::Color::Theme(6,3)),

            delta_invalid: delta.clone(),
            delta_faster: delta.clone()
                .set_background_color(xls::Color::Theme(6,2)),

            super_rally: stage_time.clone()
                .set_background_color(xls::Color::Theme(5,3)),

            driver_names: xls::Format::new()
                .set_border(xls::FormatBorder::Medium)
                .set_bold(),

            stage_time,
            delta,
        }
    }
}

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

#[derive(Debug)]
pub struct SpreadSheetError {
    message: String
}

impl SpreadSheetError {
    pub fn new(msg: String) -> Self {
        Self {
            message: msg,
        }
    }
}

impl fmt::Display for SpreadSheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpreadSheetError: {}", self.message)
    }
}

impl Error for SpreadSheetError {}

pub fn build_spreadsheet(rally: &structures::Rally, uids: &UidMap, driver: usize, benchmarks: &[usize]) -> Result<xls::Workbook, Box<dyn Error>> {

    let formats = format::get_formats();
    let driver = rally.entries.iter()
        .filter(|x| x.number == driver)
        .next()
        .ok_or_else(|| Box::new(SpreadSheetError::new(format!("Driver {} did not race in {}", driver, rally.title))))?;
    let benchmarks: Vec<_> = rally.entries.iter().filter(|x| benchmarks.contains(&x.number)).collect();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Welp time to do a bunch of bookkeeping!
    let stage_start_row = 2;
    // Title/Stage names columns
    worksheet.set_column_width(0, 18)?;
    worksheet.write_with_format(0, 0, &rally.title, &formats.bold)?;
    worksheet.write_with_format(1, 0, "Stage Name", &formats.heading)?;
    worksheet.write_with_format(1, 1, "Length", &formats.heading)?;

    // Milage column
    worksheet.set_column_width(1, 8)?;

    let format_time = |time: &structures::StageTime, overall_win: &Option<structures::StageTime>, category_class_win: &Option<structures::StageTime>| {
        if !time.is_valid() {
            return &formats.invalid_time;
        } else if Some(time) == overall_win.as_ref() {
            return &formats.overall_class_win;
        } else if Some(time) == category_class_win.as_ref() {
            return &formats.class_win;
        } else {
            return &formats.stage_time;
        }
    };

    let format_delta = |delta: structures::Delta| {
        match delta.kind {
            structures::DeltaKind::Invalid |
                structures::DeltaKind::Equal => &formats.delta_invalid,
            structures::DeltaKind::Faster => &formats.delta_faster,
            structures::DeltaKind::Slower => &formats.delta
        }
    };

    let driver_column = 2;
    let benchmark_start_column = 4;

    worksheet.write_with_format(1, driver_column,
        format!("{}", driver.number), // TODO(richo) Do the uid lookup thing to figure out who we are
        &formats.heading)?;
    for (i, benchmark) in benchmarks.iter().enumerate() {
        let bm_driver = &uids[&benchmark.driverUID];
        let driver_last_name = bm_driver.last_name();
        let bm_codriver = &uids[&benchmark.codriverUID];
        let codriver_last_name = bm_codriver.last_name();

        worksheet.merge_range(0, benchmark_start_column + (i *2 ) as u16,
                              0, benchmark_start_column + 1 + (i * 2) as u16,
            &format!("{}/{}", driver_last_name, codriver_last_name),
            &formats.driver_names)?;
        worksheet.write_with_format(1, benchmark_start_column + (i * 2) as u16,
            format!("{}", benchmark.number),
            &formats.heading)?;
        worksheet.write_with_format(1, benchmark_start_column + 1 + (i * 2) as u16,
            "Diff s/mi",
            &formats.heading)?;
    }

    for (stage_number, stage) in rally.stages.iter().enumerate() {
        worksheet.write_with_format(stage_start_row + stage_number as u32, 0, &stage.name, &formats.stage_name)?;
        worksheet.write_with_format(stage_start_row + stage_number as u32, 1, stage.length, &formats.stage_length)?;

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

        let mut fmt = format_time(&driver_time, &overall_win, &class_win);
        if driver.colors[stage_number] == structures::BoxColor::Red {
            fmt = &formats.super_rally;
        }
        worksheet.write_with_format(stage_start_row + stage_number as u32, driver_column, driver_time.to_string(), fmt)?;

        for (i, benchmark) in benchmarks.iter().enumerate() {
            let benchmark_time = benchmark.times[stage_number];
            let mut fmt = format_time(&benchmark_time, &overall_win, &class_win);
            if benchmark.colors[stage_number] == structures::BoxColor::Red {
                fmt = &formats.super_rally;
            }
            worksheet.write_with_format(stage_start_row + stage_number as u32,
                benchmark_start_column + (i * 2) as u16,
                benchmark_time.to_string(),
                fmt)?;
            if benchmark_time.is_valid() && driver_time.is_valid() {
                let delta = driver_time.diff_per_mile(&benchmark_time, stage.length);
                worksheet.write_with_format(stage_start_row + stage_number as u32,
                    benchmark_start_column + 1 + (i * 2) as u16,
                    delta.to_string(),
                    format_delta(delta))?;
            }
        }
    }

    Ok(workbook)
}
