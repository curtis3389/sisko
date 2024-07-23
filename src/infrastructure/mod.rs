use std::time::Duration;

pub mod acoustid;
pub mod musicbrainz;

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub type Am<T> = Arc<Mutex<T>>;
pub type Ram<T> = Result<Am<T>>;

pub trait DurationExtensions {
    fn to_pretty_string(&self) -> String;
}

impl DurationExtensions for Duration {
    fn to_pretty_string(&self) -> String {
        let seconds = self.as_secs();
        let hours = seconds / 3600;
        let seconds = seconds % 3600;
        let minutes = seconds / 60;
        let minute_part = match hours {
            0 => format!("{}", minutes),
            _ => format!("{:02}", minutes),
        };
        let seconds = seconds % 60;
        let hour_part = match hours {
            0 => String::new(),
            _ => format!("{}:", hours),
        };
        format!("{}{}:{:02}", hour_part, minute_part, seconds)
    }
}
