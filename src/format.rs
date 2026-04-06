pub const RST: &str = "\x1b[0m";
pub const CYAN: &str = "\x1b[36m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";

pub fn format_token_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}k", n / 1_000)
    } else {
        n.to_string()
    }
}

pub fn build_progress_bar(pct: u8, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let empty = width - filled;
    "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty)
}

pub fn get_percent_color(pct: u8) -> &'static str {
    if pct < 50 {
        GREEN
    } else if pct < 80 {
        YELLOW
    } else {
        RED
    }
}

pub fn format_countdown(resets_at: &str) -> String {
    let epoch = match iso8601_to_epoch_secs(resets_at) {
        Some(e) => e,
        None => return String::new(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff_sec = epoch - now;
    if diff_sec <= 0 {
        return "now".to_string();
    }
    let days = diff_sec / 86400;
    let hours = (diff_sec % 86400) / 3600;
    let mins = (diff_sec % 3600) / 60;
    if days > 0 {
        format!("\u{21bb}{}d{}h", days, hours)
    } else if hours > 0 {
        format!("\u{21bb}{}h{:02}m", hours, mins)
    } else {
        format!("\u{21bb}{}m", mins)
    }
}

/// Parse ISO 8601 datetime string to epoch seconds.
/// Handles: "2026-04-04T20:00:00.104774+00:00", "2026-04-04T20:00:00Z"
pub(crate) fn iso8601_to_epoch_secs(s: &str) -> Option<i64> {
    if s.len() < 19 {
        return None;
    }
    let year: i64 = s.get(0..4)?.parse().ok()?;
    let month: u32 = s.get(5..7)?.parse().ok()?;
    let day: u32 = s.get(8..10)?.parse().ok()?;
    let hour: i64 = s.get(11..13)?.parse().ok()?;
    let min: i64 = s.get(14..16)?.parse().ok()?;
    let sec: i64 = s.get(17..19)?.parse().ok()?;

    // Parse timezone offset (skip fractional seconds first)
    let rest = &s[19..];
    let tz_start = rest
        .find(|c: char| c == 'Z' || c == '+' || c == '-')
        .unwrap_or(rest.len());
    let tz_part = &rest[tz_start..];

    let offset_secs: i64 = if tz_part.is_empty() || tz_part.starts_with('Z') {
        0
    } else {
        let sign: i64 = if tz_part.starts_with('-') { -1 } else { 1 };
        let tz_body = &tz_part[1..];
        let tz_h: i64 = tz_body.get(0..2)?.parse().ok()?;
        let tz_m: i64 = if tz_body.len() >= 5 && tz_body.as_bytes()[2] == b':' {
            tz_body.get(3..5)?.parse().ok()?
        } else if tz_body.len() >= 4 {
            tz_body.get(2..4)?.parse().ok()?
        } else {
            0
        };
        sign * (tz_h * 3600 + tz_m * 60)
    };

    let days = days_from_civil(year, month, day);
    Some(days * 86400 + hour * 3600 + min * 60 + sec - offset_secs)
}

/// Howard Hinnant's algorithm: civil date to days since Unix epoch.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_token_count() {
        assert_eq!(format_token_count(0), "0");
        assert_eq!(format_token_count(500), "500");
        assert_eq!(format_token_count(1_000), "1k");
        assert_eq!(format_token_count(1_500), "1k");
        assert_eq!(format_token_count(200_000), "200k");
        assert_eq!(format_token_count(1_000_000), "1.0M");
        assert_eq!(format_token_count(1_500_000), "1.5M");
    }

    #[test]
    fn test_build_progress_bar() {
        let bar = build_progress_bar(50, 10);
        assert_eq!(bar, "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}");

        let bar = build_progress_bar(0, 10);
        assert_eq!(bar, "\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}");

        let bar = build_progress_bar(100, 10);
        assert_eq!(bar, "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}");
    }

    #[test]
    fn test_get_percent_color() {
        assert_eq!(get_percent_color(0), GREEN);
        assert_eq!(get_percent_color(49), GREEN);
        assert_eq!(get_percent_color(50), YELLOW);
        assert_eq!(get_percent_color(79), YELLOW);
        assert_eq!(get_percent_color(80), RED);
        assert_eq!(get_percent_color(100), RED);
    }

    #[test]
    fn test_iso8601_to_epoch_secs() {
        // 1970-01-01T00:00:00Z = epoch 0
        assert_eq!(iso8601_to_epoch_secs("1970-01-01T00:00:00Z"), Some(0));

        // With timezone offset
        assert_eq!(
            iso8601_to_epoch_secs("1970-01-01T01:00:00+01:00"),
            Some(0)
        );

        // With fractional seconds
        assert!(iso8601_to_epoch_secs("2026-04-04T20:00:00.104774+00:00").is_some());

        // Invalid input
        assert_eq!(iso8601_to_epoch_secs("bad"), None);
        assert_eq!(iso8601_to_epoch_secs(""), None);
    }

    #[test]
    fn test_format_countdown_empty() {
        assert_eq!(format_countdown(""), String::new());
        assert_eq!(format_countdown("invalid"), String::new());
    }

    #[test]
    fn test_format_countdown_days() {
        let result = format_countdown("2099-01-04T00:00:00+00:00");
        assert!(result.contains('d'), "expected days in: {result}");
        assert!(result.starts_with('\u{21bb}'));
    }

    #[test]
    fn test_format_countdown_past() {
        assert_eq!(format_countdown("2020-01-01T00:00:00+00:00"), "now");
    }
}
