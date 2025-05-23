use serde::Deserialize;
use std::time;
use std::sync::OnceLock;
use regex::Regex;
use serde::de::{self, Visitor, Deserializer};
use std::fmt;
use std::collections::HashMap;

pub type UidMap = HashMap::<usize, Uid>;

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
     #[serde(rename(deserialize = "Rally Ready RallySprint"))]
    RallyReadyRallySprint,
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

#[derive(Deserialize, Clone, PartialEq)]
pub enum BoxColor {
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub struct StageTime {
    time: time::Duration,
}

impl std::ops::Add for StageTime {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            time: self.time + other.time,
        }
    }
}

impl std::ops::Sub for StageTime {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            time: self.time - other.time,
        }
    }
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

    pub fn zero() -> Self {
        Self {
            time: std::time::Duration::ZERO,
        }
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
        pub driverUID: usize,
        pub codriverUID: usize,
        #[serde(rename(deserialize = "carClass"))]
        pub class: Class,
        #[serde(rename(deserialize = "carModel"))]
        model: String,
        pub times: Vec<StageTime>,
        pub colors: Vec<BoxColor>,
        penalties: Vec<Penalty>,
        retirements: Vec<Retirement>,
        pub splits: Option<Vec<Vec<StageTime>>>,
}

impl Entry {
    pub fn driver<'a>(&self, map: &'a UidMap) -> &'a Uid {
        &map[&self.driverUID]
    }

    pub fn codriver<'a>(&self, map: &'a UidMap) -> &'a Uid {
        &map[&self.codriverUID]
    }

    pub fn names(&self, map: &UidMap) -> String {
        format!("{}/{}", self.driver(&map).l, self.codriver(&map).l)
    }

    /// This is the cumulative time to this split
    pub fn splits_with_finish(&self) -> Vec<Vec<StageTime>> {
        // TODO(richo) There's some clever way to do this with once and chain but I'm tired
        let mut splits = self.splits.clone().unwrap_or_else(|| {
            let mut vec = vec![];
            for _ in 0..self.times.len() {
                vec.push(vec![])
            };
            vec
        });
        for (splits, time) in splits.iter_mut().zip(self.times.iter()) {
            splits.push(*time);
        }
        splits
    }

    /// The sector time in this split
    pub fn sectors_with_finish(&self) -> Vec<Vec<StageTime>> {
        let mut sectors = self.splits_with_finish();
        println!("{:?}", &sectors);
        for stage in sectors.iter_mut() {
            let mut prev_time = StageTime::zero();
            for time in stage.iter_mut() {
                if ! time.is_valid() {
                    continue
                }
                let elapsed = *time - prev_time;
                *time = elapsed;
                prev_time = elapsed;
            }
        }

        sectors
    }
}


#[derive(Deserialize, Clone)]
pub struct Stage {
    pub name: String,
    pub length: f32,
    pub splits: Option<Vec<f32>>,
}

impl Stage {
    pub fn has_splits(&self) -> bool {
        match &self.splits {
            Some(n) => n.len() > 0,
            None => false,
        }
    }

    pub fn splits_with_finish(&self) -> Vec<f32> {
        // TODO(richo) There's some clever way to do this with once and chain but I'm tired
        let mut splits = self.splits.clone().unwrap_or_else(Vec::new);
        splits.push(self.length);
        splits
    }
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

impl Uid {
    pub fn first_name(&self) -> &str {
        &self.f
    }

    pub fn last_name(&self) -> &str {
        &self.l
    }
}
