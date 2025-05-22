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

    impl Formats {
        pub fn delta(&self, delta: crate::structures::Delta) -> &xls::Format {
            match delta.kind {
                crate::structures::DeltaKind::Invalid |
                    crate::structures::DeltaKind::Equal => &self.delta_invalid,
                crate::structures::DeltaKind::Faster => &self.delta_faster,
                crate::structures::DeltaKind::Slower => &self.delta
            }
        }
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

pub fn build_stage_with_splits(rally: &structures::Rally, uids: &UidMap, driver: &structures::Entry, benchmarks: &Vec<&structures::Entry>, stage: &structures::Stage, stage_index: usize, sheet: &mut xls::Worksheet) -> Result<(), Box<dyn Error>> {
    let formats = format::get_formats();

    // Other way around, we'll do drivers per row
    sheet.set_column_width(0, 18)?;
    sheet.write_with_format(0, 0, &stage.name, &formats.bold)?;
    sheet.write_with_format(0, 1, "Length", &formats.heading)?;
    sheet.write_with_format(0, 2, stage.length, &formats.stage_length)?;

    sheet.write_with_format(1, 0, "Team", &formats.heading)?;

    let stage_splits = stage.splits_with_finish();
    for (i, split) in stage_splits.iter().enumerate() {
        let col = ((1+i) * 3) as u16;
        sheet.write_with_format(1, col,
            *split,
            &formats.stage_length)?;
        sheet.write_with_format(1, col+1,
            "Diff s/mi",
            &formats.stage_name)?;

        sheet.write_with_format(1, col+2,
            "Cumulative s/mi",
            &formats.stage_name)?;
    }



    let name_column = 0;
    let driver_row = 2;
    sheet.write_with_format(driver_row, name_column,
        driver.names(&uids),
        &formats.heading)?;
    let driver_splits = &driver.splits_with_finish()[stage_index];
    let driver_sectors = &driver.sectors_with_finish()[stage_index];
    for (n, split) in driver_splits.iter().enumerate() {
        sheet.write_with_format(driver_row,
            (n*3 + 3) as u16,
            split.to_string(),
            &formats.stage_time)?;
    }

    let benchmark_start = 3;
    for (row, bm) in benchmarks.iter().enumerate() {
        let row = (row + benchmark_start) as u32;
        sheet.write_with_format(
            row,
            name_column,
            bm.names(&uids),
            &formats.bold)?;
        let bm_splits = &bm.splits_with_finish()[stage_index];
        let bm_sectors = &bm.sectors_with_finish()[stage_index];
        let mut prev_split_distance = 0.0;
        for (n, ((split, sector), split_distance)) in bm_splits.iter().zip(bm_sectors.iter()).zip(stage_splits.iter()).enumerate() {
            let driver_split = driver_splits[n];
            let driver_sector = driver_sectors[n];
            let col = (n*3 + 3) as u16;
            sheet.write_with_format(row,
                col,
                split.to_string(),
                &formats.stage_time)?;

            if split.is_valid() && driver_split.is_valid() {
                let cumulative_delta = driver_split.diff_per_mile(&split, *split_distance);

                let this_sector = split_distance - prev_split_distance;
                let sector_delta = driver_sector.diff_per_mile(&sector, this_sector);
                sheet.write_with_format(row,
                    col+1,
                    sector_delta.to_string(),
                    formats.delta(sector_delta))?;
                sheet.write_with_format(row,
                    col+2,
                    cumulative_delta.to_string(),
                    formats.delta(cumulative_delta))?;
            }
            prev_split_distance = *split_distance;
        }
    }
    Ok(())
}

pub fn build_overview(rally: &structures::Rally, uids: &UidMap, driver: &structures::Entry, benchmarks: &Vec<&structures::Entry>,  sheet: &mut xls::Worksheet) -> Result<(), Box<dyn Error>> {
    let formats = format::get_formats();

    let stage_start_row = 2;
    // Title/Stage names columns
    sheet.set_column_width(0, 18)?;
    sheet.write_with_format(0, 0, &rally.title, &formats.bold)?;
    sheet.write_with_format(1, 0, "Stage Name", &formats.heading)?;
    sheet.write_with_format(1, 1, "Length", &formats.heading)?;

    // Milage column
    sheet.set_column_width(1, 8)?;

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

    let driver_column = 2;
    let benchmark_start_column = 4;

    sheet.write_with_format(1, driver_column,
        format!("{}", driver.number), // TODO(richo) Do the uid lookup thing to figure out who we are
        &formats.heading)?;
    for (i, benchmark) in benchmarks.iter().enumerate() {

        sheet.merge_range(0, benchmark_start_column + (i *2 ) as u16,
                              0, benchmark_start_column + 1 + (i * 2) as u16,
                              &benchmark.names(&uids),
            &formats.driver_names)?;
        sheet.write_with_format(1, benchmark_start_column + (i * 2) as u16,
            format!("{}", benchmark.number),
            &formats.heading)?;
        sheet.write_with_format(1, benchmark_start_column + 1 + (i * 2) as u16,
            "Diff s/mi",
            &formats.heading)?;
    }

    for (stage_number, stage) in rally.stages.iter().enumerate() {
        sheet.write_with_format(stage_start_row + stage_number as u32, 0, &stage.name, &formats.stage_name)?;
        sheet.write_with_format(stage_start_row + stage_number as u32, 1, stage.length, &formats.stage_length)?;

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
        sheet.write_with_format(stage_start_row + stage_number as u32, driver_column, driver_time.to_string(), fmt)?;

        for (i, benchmark) in benchmarks.iter().enumerate() {
            let benchmark_time = benchmark.times[stage_number];
            let mut fmt = format_time(&benchmark_time, &overall_win, &class_win);
            if benchmark.colors[stage_number] == structures::BoxColor::Red {
                fmt = &formats.super_rally;
            }
            sheet.write_with_format(stage_start_row + stage_number as u32,
                benchmark_start_column + (i * 2) as u16,
                benchmark_time.to_string(),
                fmt)?;
            if benchmark_time.is_valid() && driver_time.is_valid() {
                let delta = driver_time.diff_per_mile(&benchmark_time, stage.length);
                sheet.write_with_format(stage_start_row + stage_number as u32,
                    benchmark_start_column + 1 + (i * 2) as u16,
                    delta.to_string(),
                    formats.delta(delta))?;
            }
        }
    }

    Ok(())
}

pub fn build_spreadsheet(rally: &structures::Rally, uids: &UidMap, driver: usize, benchmarks: &[usize]) -> Result<xls::Workbook, Box<dyn Error>> {

    let driver = rally.entries.iter()
        .filter(|x| x.number == driver)
        .next()
        .ok_or_else(|| Box::new(SpreadSheetError::new(format!("Driver {} did not race in {}", driver, rally.title))))?;
    let benchmarks: Vec<_> = rally.entries.iter().filter(|x| benchmarks.contains(&x.number)).collect();

    let mut workbook = Workbook::new();
    let mut overview = workbook.add_worksheet();
    overview.set_name(&rally.slug)?;
    build_overview(&rally, &uids, &driver, &benchmarks, &mut overview)?;


    for (stage_number, stage) in rally.stages.iter().enumerate() {
        if !stage.has_splits() {
            continue
        }
        let mut split_sheet = workbook.add_worksheet();
        split_sheet.set_name(format!("SS{} {}", stage_number+1, &stage.name))?;
        build_stage_with_splits(&rally, &uids, &driver, &benchmarks, &stage, stage_number, &mut split_sheet)?;
    }

    Ok(workbook)
}
