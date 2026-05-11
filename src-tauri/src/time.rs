//! Converts system and millisecond timestamps to RFC 3339 UTC strings.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn ms_to_rfc3339(millis: i64) -> String {
    system_time_to_rfc3339(UNIX_EPOCH + std::time::Duration::from_millis(millis.max(0) as u64))
}

pub fn system_time_to_rfc3339(value: SystemTime) -> String {
    let duration = value.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = duration.as_secs() as i64;

    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let second = (secs_of_day % 60) as u32;

    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_rfc3339_clamps_negative_to_epoch() {
        assert_eq!(ms_to_rfc3339(-1), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn system_time_to_rfc3339_formats_epoch() {
        assert_eq!(system_time_to_rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn civil_from_days_formats_epoch_day() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }
}
