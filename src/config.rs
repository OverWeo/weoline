use std::path::PathBuf;

pub enum Mode {
    Full,
    Compact,
    Minimal,
}

#[allow(dead_code)]
pub struct Config {
    pub mode: Mode,
    pub show_tokens: bool,
    pub show_cache: bool,
    pub show_session: bool,
    pub show_limits: bool,
    pub show_weekly: bool,
    pub show_sonnet: bool,
    pub show_5h_timer: bool,
    pub show_weekly_timer: bool,
    pub show_sonnet_timer: bool,
    pub refresh_interval: u64,
    pub bar_width: usize,
    pub credentials_file: PathBuf,
    pub cache_file: PathBuf,
}

#[cfg(feature = "usage-tracking")]
pub const STALE_MULTIPLIER: u64 = 3;

impl Config {
    pub fn from_env() -> Self {
        let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);

        Config {
            mode: match std::env::var("SL_MODE").as_deref() {
                Ok("compact") => Mode::Compact,
                Ok("minimal") => Mode::Minimal,
                _ => Mode::Full,
            },
            show_tokens: std::env::var("SL_SHOW_TOKENS").as_deref() == Ok("1"),
            show_cache: std::env::var("SL_SHOW_CACHE").as_deref() != Ok("0"),
            show_session: std::env::var("SL_SHOW_SESSION").as_deref() != Ok("0"),
            show_limits: std::env::var("SL_SHOW_LIMITS").as_deref() != Ok("0"),
            show_weekly: std::env::var("SL_SHOW_WEEKLY").as_deref() != Ok("0"),
            show_sonnet: std::env::var("SL_SHOW_SONNET").as_deref() != Ok("0"),
            show_5h_timer: std::env::var("SL_SHOW_5H_TIMER").as_deref() != Ok("0"),
            show_weekly_timer: std::env::var("SL_SHOW_WEEKLY_TIMER").as_deref() == Ok("1"),
            show_sonnet_timer: std::env::var("SL_SHOW_SONNET_TIMER").as_deref() == Ok("1"),
            refresh_interval: std::env::var("SL_REFRESH_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            bar_width: std::env::var("SL_BAR_WIDTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(14),
            credentials_file: std::env::var("SL_CREDENTIALS_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".claude").join(".credentials.json")),
            cache_file: std::env::var("SL_CACHE_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".claude").join("usage-cache.json")),
        }
    }
}
