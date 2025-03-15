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
    nosecondsre: Regex,
}

fn parse_stage_time<'a>(time: &'a str) -> Option<StageTime> {
    static REGEX: OnceLock<StageTimeRegexes> = OnceLock::new();
    let regexes = REGEX.get_or_init(|| {
        StageTimeRegexes {
            hoursre: Regex::new(r"^(\d+):(\d+):(\d+\.\d+)$").unwrap(),
            minutesre: Regex::new(r"^(\d+):(\d+\.\d+)$").unwrap(),
            secondsre: Regex::new(r"^(\d+\.\d+)$").unwrap(),
            nosecondsre: Regex::new(r"^(\d+):(\d+)$").unwrap(),
        }
    });

    // We really want to make this be an option but that has.. annoying type implications
    if time == "" {
        return Some(StageTime { time: time::Duration::new(0, 0) })
    }

    for (_, [hours, minutes, seconds]) in regexes.hoursre.captures_iter(time).map(|c| c.extract()) {
        let hours: u64 = hours.parse().unwrap();
        let minutes: u64 = minutes.parse().unwrap();
        let seconds: f32 = seconds.parse().unwrap();
        // Make sure the tenths are correct
        let millis = (seconds.fract() * 10f32).round() as u32;
        return Some(StageTime { time: time::Duration::new(
            (hours * 60 * 60) +
            (minutes * 60) +
            seconds as u64,
            millis * 1_000_000_00
        ) } )
    }
    for (_, [minutes, seconds]) in regexes.minutesre.captures_iter(time).map(|c| c.extract()) {
        let minutes: u64 = minutes.parse().unwrap();
        let seconds: f32 = seconds.parse().unwrap();
        // Make sure the tenths are correct
        let millis = (seconds.fract() * 10f32).round() as u32;
        return Some(StageTime { time: time::Duration::new(
            (minutes * 60) +
            seconds as u64,
            millis * 1_000_000_00
        ) } )
    }
    for (_, [seconds]) in regexes.secondsre.captures_iter(time).map(|c| c.extract()) {
        let seconds: f32 = seconds.parse().unwrap();
        // Make sure the tenths are correct
        let millis = (seconds.fract() * 10f32).round() as u32;
        return Some(StageTime { time: time::Duration::new(
            seconds as u64,
            millis * 1_000_000_00
        ) } )
    }
    for (_, [minutes, seconds]) in regexes.nosecondsre.captures_iter(time).map(|c| c.extract()) {
        let minutes: u64 = minutes.parse().unwrap();
        let seconds: f32 = seconds.parse().unwrap();
        return Some(StageTime { time: time::Duration::new(
            (minutes * 60) +
            seconds as u64,
            0
        ) } )
    }
    return None
}

#[derive(Deserialize, Clone)]
pub struct Rally {
    source: String,
    startDate: String,
    finishDate: String,
    pub title: String,
    pub slug: String,
    pub entries: Vec<Entry>,
    pub stages: Vec<Stage>,
}

impl Rally {
    pub fn entry_by_driver_number<'a>(&'a self, number: usize) -> Option<&'a Entry> {
        self.entries.iter().filter(|i| i.number == number).next()
    }
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub enum Category {
    National,
    Regional,
    RallySprint,
    Exhibition,
}
#[derive(Deserialize, Copy, Clone, PartialEq)]
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

#[derive(Deserialize, Clone)]
enum BoxColor {
    // TODO(richo) Yeah I dunno what this is honestly.
    #[serde(rename(deserialize = ""))]
    None,
    #[serde(rename(deserialize = "red"))]
    Red,
}

#[derive(Deserialize, Clone)]
struct Penalty {
}

#[derive(Deserialize, Clone)]
enum RetirementStatus {
    Permanent,
    Temporary,
    Rejoined,
}

#[derive(Deserialize, Clone)]
struct Retirement {
    control: String,
    stage: usize,
    status: RetirementStatus,
    reason: String,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub struct StageTime {
    time: time::Duration,
}

impl StageTime {
    pub fn is_valid(&self) -> bool {
        !self.time.is_zero()
    }

    pub fn diff_per_mile(&self, other: &Self, distance: f32) -> Delta {
        if ! (self.is_valid() && other.is_valid()) {
            return Delta::invalid();
        }

        if self > other {
            return Delta {
                delta: (self.time - other.time).as_secs_f32() / distance,
                kind: DeltaKind::Slower,
            }
        } else if other > self {
            return Delta {
                delta: (other.time - self.time).as_secs_f32() / distance,
                kind: DeltaKind::Faster,
            }
        }
        return Delta::equal();
    }
}

pub enum DeltaKind {
    Faster,
    Slower,
    Equal,
    Invalid,
}

pub struct Delta {
    pub delta: f32,
    pub kind: DeltaKind,
}

impl Delta {
    fn invalid() -> Self {
        Delta {
            delta: 0.0,
            kind: DeltaKind::Invalid,
        }
    }

    fn equal() -> Self {
        Delta {
            delta: 0.0,
            kind: DeltaKind::Equal,
        }
    }
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sign = match self.kind {
            DeltaKind::Invalid |
                DeltaKind::Equal |
                DeltaKind::Faster => "",
            DeltaKind::Slower => "-",
        };
        return write!(f, "{}{:.02}", sign, self.delta)
    }
}

impl fmt::Display for StageTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.time.as_secs();
        let millis = self.time.subsec_millis() ;

        let hours = secs / 3600;
        let secs = secs % 3600;

        let mins = secs / 60;
        let secs = secs % 60;

        if hours > 0 {
            return write!(f, "{}:{:02}:{:02}.{:02}", hours, mins, secs, millis);
        }
        if mins > 0 {
            return write!(f, "{:02}:{:02}.{:02}", mins, secs, millis);
        }
        write!(f, "{:02}.{:02}", secs, millis)
    }
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


#[derive(Deserialize, Clone)]
pub struct Entry {
        pub category: Category,
        pub number: usize,
        driverUID: usize,
        codriverUID: usize,
        #[serde(rename(deserialize = "carClass"))]
        pub class: Class,
        #[serde(rename(deserialize = "carModel"))]
        model: String,
        pub times: Vec<StageTime>,
        colors: Vec<BoxColor>,
        penalties: Vec<Penalty>,
        retirements: Vec<Retirement>,
}

#[derive(Deserialize, Clone)]
pub struct Stage {
    pub name: String,
    pub length: f32,
}

#[derive(Deserialize, Clone)]
pub struct Uid {
    pub uid: usize,
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
