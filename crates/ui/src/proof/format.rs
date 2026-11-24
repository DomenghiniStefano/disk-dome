//! Number and date formatting for the proof's cells. Temporary: the real formatting (locale, units) arrives
//! with the design of the table.

use std::fmt::Write;

/// Size with binary units, as Explorer shows it: "512 B", "1.5 KB", "12.3 GB".
pub fn size(bytes: u64, out: &mut String) {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        let _ = write!(out, "{bytes} B");
        return;
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    let _ = write!(out, "{value:.1} {}", UNITS[unit]);
}

/// Share of the parent with one decimal: "12.3 %".
pub fn percent(part: u64, whole: u64, out: &mut String) {
    let share = if whole == 0 {
        0.0
    } else {
        part as f64 * 100.0 / whole as f64
    };
    let _ = write!(out, "{share:.1} %");
}

/// Unix seconds as "YYYY-MM-DD HH:MM", UTC.
pub fn date(unix_seconds: u32, out: &mut String) {
    let seconds = i64::from(unix_seconds);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let minute_of_day = seconds.rem_euclid(86_400) / 60;
    let _ = write!(
        out,
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        minute_of_day / 60,
        minute_of_day % 60
    );
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{date, percent, size};

    fn run(f: impl FnOnce(&mut String)) -> String {
        let mut out = String::new();
        f(&mut out);
        out
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(run(|o| size(0, o)), "0 B");
        assert_eq!(run(|o| size(1023, o)), "1023 B");
        assert_eq!(run(|o| size(1536, o)), "1.5 KB");
        assert_eq!(run(|o| size(5 * 1024 * 1024 * 1024, o)), "5.0 GB");
    }

    #[test]
    fn formats_percent() {
        assert_eq!(run(|o| percent(1, 8, o)), "12.5 %");
        assert_eq!(run(|o| percent(1, 0, o)), "0.0 %");
    }

    #[test]
    fn formats_dates() {
        assert_eq!(run(|o| date(0, o)), "1970-01-01 00:00");
        assert_eq!(run(|o| date(951_782_400, o)), "2000-02-29 00:00");
        assert_eq!(run(|o| date(1_764_000_000, o)), "2025-11-24 16:00");
    }
}
