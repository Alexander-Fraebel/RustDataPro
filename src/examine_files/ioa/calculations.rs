use crate::data::Timeline;
use egui::Key;

/// Used for Total Count and Total Duration IOA. Divides the smaller value by the larger value. If both values are zero returns None.
pub fn single_pair_total_ratio_ioa(primary: f32, reli: f32) -> Option<f32> {
    if primary == 0.0 && reli == 0.0 {
        return None;
    }
    if primary >= reli {
        Some(reli / primary)
    } else {
        Some(primary / reli)
    }
}

pub fn extract_times(v: &Timeline, key: Key) -> Vec<f32> {
    v.iter().filter(|e| e.0 == key).map(|e| e.1).collect()
}

/// Caclulate the fraction of intervals in which both primary and reliability data have the same count. If no intervals exist returns None.
/// If strict is true then any intervals in which neither data set records anything are ignored from the total.
pub fn single_pair_interval_ioa(
    max_time: f32,
    interval: f32,
    key: Key,
    primary: &Timeline,
    reli: &Timeline,
    strict: bool,
) -> Option<f32> {
    let mut time = interval;

    let primary = extract_times(primary, key);
    let mut p_iter = primary.into_iter().peekable();
    let reli = extract_times(reli, key);
    let mut r_iter = reli.into_iter().peekable();

    let mut correct_intervals = 0.0;
    let mut total_intervals = 0.0;
    while time <= max_time {
        let mut pctr = 0.0;
        while p_iter.next_if(|x| x <= &time).is_some() {
            pctr += 1.0;
        }
        let mut rctr = 0.0;
        while r_iter.next_if(|x| x <= &time).is_some() {
            rctr += 1.0;
        }
        if strict && pctr == 0.0 && rctr == 0.0 {
            // In strict mode ignore intervals when primary and reli both scored nothing
        } else {
            if pctr == rctr {
                correct_intervals += 1.0;
            }
            total_intervals += 1.0;
        }

        time += interval;
    }

    if total_intervals == 0.0 {
        None
    } else {
        Some(correct_intervals / total_intervals)
    }
}
