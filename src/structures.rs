use serde::Deserialize;
use std::time;
use std::sync::OnceLock;
use regex::Regex;
use serde::de::{self, Visitor, Deserializer};
use std::fmt;

struct StageTimeRegexes {
    hoursre: Regex,
    minutesre: Regex,
    secondsre: Regex,
}

fn parse_stage_time<'a>(time: &'a str) -> Option<StageTime> {
    static REGEX: OnceLock<StageTimeRegexes> = OnceLock::new();
    let regexes = REGEX.get_or_init(|| {
        StageTimeRegexes {
            hoursre: Regex::new(r"^(\d+):(\d+):(\d+).(\d+)$").unwrap(),
            minutesre: Regex::new(r"^(\d+):(\d+).(\d+)$").unwrap(),
            secondsre: Regex::new(r"^(\d+).(\d+)$").unwrap(),
        }
    });

    // We really want to make this be an option but that has.. annoying type implications
    if time == "" {
        return Some(StageTime { time: time::Duration::new(0, 0) })
    }

    for (_, [hours, minutes, seconds, millis]) in regexes.hoursre.captures_iter(time).map(|c| c.extract()) {
        let hours: u64 = hours.parse().unwrap();
        let minutes: u64 = minutes.parse().unwrap();
        let seconds: u64 = seconds.parse().unwrap();
        let millis: u32 = millis.parse().unwrap();
        return Some(StageTime { time: time::Duration::new(
            (hours * 60 * 60) +
            (minutes * 60) +
            seconds,
            millis * 1000
        ) } )
    }
    for (_, [minutes, seconds, millis]) in regexes.minutesre.captures_iter(time).map(|c| c.extract()) {
        let minutes: u64 = minutes.parse().unwrap();
        let seconds: u64 = seconds.parse().unwrap();
        let millis: u32 = millis.parse().unwrap();
        return Some(StageTime { time: time::Duration::new(
            (minutes * 60) +
            seconds,
            millis * 1000
        ) } )
    }
    for (_, [seconds, millis]) in regexes.secondsre.captures_iter(time).map(|c| c.extract()) {
        let seconds: u64 = seconds.parse().unwrap();
        let millis: u32 = millis.parse().unwrap();
        return Some(StageTime { time: time::Duration::new(
            seconds,
            millis * 1000
        ) } )
    }
    return None
}

#[derive(Deserialize)]
pub struct Rally {
    source: String,
    startDate: String,
    finishDate: String,
    title: String,
    pub slug: String,
    entries: Vec<Entry>,
    stages: Vec<Stage>,
}

impl Rally {
    pub fn entry_by_driver_number<'a>(&'a self, number: usize) -> Option<&'a Entry> {
        self.entries.iter().filter(|i| i.number == number).next()
    }
}

#[derive(Deserialize)]
enum Category {
    National,
    Regional,
    RallySprint,
    Exhibition,
}
#[derive(Deserialize, Copy, Clone)]
pub enum Class {
    O4WD,
    L4WD,
    O2WD,
    L2WD,
    RC2,
    NA4WD,
    #[serde(rename(deserialize = "Class-X"))]
    #[serde(rename(deserialize = "Class X"))]
    ClassX,
}

#[derive(Deserialize)]
enum BoxColor {
    // TODO(richo) Yeah I dunno what this is honestly.
    #[serde(rename(deserialize = ""))]
    None,
    #[serde(rename(deserialize = "red"))]
    Red,
}

#[derive(Deserialize)]
struct Penalty {
}

#[derive(Deserialize)]
enum RetirementStatus {
    Permanent,
    Temporary,
    Rejoined,
}

#[derive(Deserialize)]
struct Retirement {
    control: String,
    stage: usize,
    status: RetirementStatus,
    reason: String,
}

struct StageTime {
    time: time::Duration,
}

struct StageTimeVisitor;

impl<'de> Visitor<'de> for StageTimeVisitor {
    type Value = StageTime;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a stage time formatted as 0:00:00.0")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_stage_time(value).ok_or(E::custom("Invalid time"))
    }
}

impl<'de> Deserialize<'de> for StageTime {
    fn deserialize<D>(deserializer: D) -> Result<StageTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StageTimeVisitor)
    }
}


#[derive(Deserialize)]
pub struct Entry {
        category: Category,
        number: usize,
        driverUID: usize,
        codriverUID: usize,
        #[serde(rename(deserialize = "carClass"))]
        pub class: Class,
        #[serde(rename(deserialize = "carModel"))]
        model: String,
        times: Vec<StageTime>,
        colors: Vec<BoxColor>,
        penalties: Vec<Penalty>,
        retirements: Vec<Retirement>,
}

#[derive(Deserialize)]
struct Stage {
    name: String,
    length: f32,
}

#[derive(Deserialize)]
pub struct Uid {
    uid: usize,
    f: String,
    l: String,
    tn: Option<String>,
    fb: Option<String>,
    ig: Option<String>,
    yt: Option<String>,
    tt: Option<String>,
    tw: Option<String>,
    web: Option<String>,
    email: Option<String>,
}
