use chrono::{DateTime, Datelike, Local, Timelike};

/// Weekday Month/Day/Year Hour:Minute
pub fn date_time_string(dt: &DateTime<Local>) -> String {
    format!(
        "{} {}/{}/{} {:02}:{:02}",
        dt.weekday(),
        dt.month(),
        dt.day(),
        dt.year(),
        dt.hour(),
        dt.minute(),
    )
}

/// Quick time stamp as YYYYMMDDhhmm
pub fn time_stamp() -> String {
    let dt = Local::now();
    format!(
        "{:04}{:02}{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
    )
}
