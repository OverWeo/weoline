use crate::config::{Config, Mode};
use crate::format::{
    build_progress_bar, format_countdown, format_token_count, get_percent_color, CYAN, RST,
};
use crate::types::{CacheData, ContextWindow, CurrentUsage, StdinData};

pub fn render(config: &Config, input: &StdinData, cache: Option<&CacheData>) -> String {
    let cw = &input.context_window;
    let usage = &cw.current_usage;

    let mut sections = Vec::new();

    if let Some(ctx) = build_context(cw, config.bar_width) {
        sections.push(ctx);
    }

    match config.mode {
        Mode::Full => {
            let mut tok_parts = Vec::new();
            if config.show_tokens {
                if let Some(tok) = build_tokens(usage) {
                    tok_parts.push(tok);
                }
            }
            if config.show_cache {
                if let Some(cch) = build_cache_section(usage) {
                    tok_parts.push(cch);
                }
            }
            if !tok_parts.is_empty() {
                sections.push(tok_parts.join("  "));
            }

            if config.show_session {
                if let Some(sess) = build_session(cw) {
                    sections.push(sess);
                }
            }

            if config.show_limits {
                if let Some(lim) = build_limits(cache, config) {
                    sections.push(lim);
                }
            }
        }
        Mode::Compact => {
            if config.show_limits {
                if let Some(lim) = build_limits(cache, config) {
                    sections.push(lim);
                }
            }
        }
        Mode::Minimal => {}
    }

    if sections.is_empty() {
        String::new()
    } else {
        sections.join("  |  ") + "\n"
    }
}

fn build_context(cw: &ContextWindow, bar_width: usize) -> Option<String> {
    let used_pct = cw.used_percentage?;
    let ctx_size = cw.context_window_size.filter(|&s| s > 0)?;

    let used_int = used_pct.round().clamp(0.0, 100.0) as u8;
    let bar = build_progress_bar(used_int, bar_width);
    let color = get_percent_color(used_int);
    let used_tokens = ((ctx_size as f64) * used_pct / 100.0).round() as u64;

    Some(format!(
        "\u{1f9e0} {color}{bar} {used_int}% ({used}/{total}){RST}",
        used = format_token_count(used_tokens),
        total = format_token_count(ctx_size),
    ))
}

fn build_tokens(usage: &CurrentUsage) -> Option<String> {
    let input = usage.input_tokens?;
    let output = usage.output_tokens.unwrap_or(0);
    Some(format!(
        "{CYAN}\u{1f4e5} in: {}  \u{1f4e4} out: {}{RST}",
        format_token_count(input),
        format_token_count(output),
    ))
}

fn build_cache_section(usage: &CurrentUsage) -> Option<String> {
    if usage.cache_read_input_tokens.is_none() && usage.cache_creation_input_tokens.is_none() {
        return None;
    }
    Some(format!(
        "{CYAN}\u{1f4be} cache read: {}  write: {}{RST}",
        format_token_count(usage.cache_read_input_tokens.unwrap_or(0)),
        format_token_count(usage.cache_creation_input_tokens.unwrap_or(0)),
    ))
}

fn build_session(cw: &ContextWindow) -> Option<String> {
    let input = cw.total_input_tokens?;
    let output = cw.total_output_tokens.unwrap_or(0);
    Some(format!(
        "{CYAN}\u{1f504} session: \u{1f4e5} in: {}  \u{1f4e4} out: {}{RST}",
        format_token_count(input),
        format_token_count(output),
    ))
}

fn build_limits(cache: Option<&CacheData>, config: &Config) -> Option<String> {
    let cache = cache?;
    let mut parts = Vec::new();

    // 5-hour limit
    if let Some(h5) = cache.five_hour.utilization {
        if h5.is_finite() && h5 >= 0.0 {
            let color = get_percent_color(h5 as u8);
            let mut s = format!("{color}\u{23f1} 5h: {h5}%{RST}");
            if config.show_5h_timer {
                if let Some(ref resets_at) = cache.five_hour.resets_at {
                    let cd = format_countdown(resets_at);
                    if !cd.is_empty() {
                        s.push(' ');
                        s.push_str(&cd);
                    }
                }
            }
            parts.push(s);
        }
    }

    // 7-day weekly
    if config.show_weekly {
        if let Some(d7) = cache.seven_day.utilization {
            if d7.is_finite() && d7 >= 0.0 {
                let color = get_percent_color(d7 as u8);
                let mut s = format!("{color}\u{1f4c5} 7d: {d7}%{RST}");
                if config.show_weekly_timer {
                    if let Some(ref resets_at) = cache.seven_day.resets_at {
                        let cd = format_countdown(resets_at);
                        if !cd.is_empty() {
                            s.push(' ');
                            s.push_str(&cd);
                        }
                    }
                }
                parts.push(s);
            }
        }
    }

    // Sonnet weekly
    if config.show_sonnet {
        if let Some(son) = cache.seven_day_sonnet.utilization {
            if son.is_finite() && son >= 0.0 {
                let color = get_percent_color(son as u8);
                let mut s = format!("{color}\u{1f3b5} sonnet: {son}%{RST}");
                if config.show_sonnet_timer {
                    if let Some(ref resets_at) = cache.seven_day_sonnet.resets_at {
                        let cd = format_countdown(resets_at);
                        if !cd.is_empty() {
                            s.push(' ');
                            s.push_str(&cd);
                        }
                    }
                }
                parts.push(s);
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    let mut result = parts.join("  ");
    if cache.is_stale {
        result.push_str(" \u{26a0}");
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextWindow, CurrentUsage, StdinData};

    fn default_config() -> Config {
        Config {
            mode: Mode::Full,
            show_tokens: true,
            show_cache: true,
            show_session: true,
            show_limits: true,
            show_weekly: true,
            show_sonnet: true,
            show_5h_timer: true,
            show_weekly_timer: false,
            show_sonnet_timer: false,
            refresh_interval: 300,
            bar_width: 14,
            credentials_file: std::path::PathBuf::new(),
            cache_file: std::path::PathBuf::new(),
        }
    }

    #[test]
    fn test_empty_input() {
        let config = default_config();
        let input = StdinData::default();
        let output = render(&config, &input, None);
        assert!(output.is_empty());
    }

    #[test]
    fn test_context_only() {
        let mut config = default_config();
        config.mode = Mode::Minimal;
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(45.0),
                context_window_size: Some(200_000),
                ..Default::default()
            },
        };
        let output = render(&config, &input, None);
        assert!(output.contains("45%"));
        assert!(output.contains("90k/200k"));
    }

    #[test]
    fn test_zero_context_size_skipped() {
        let config = default_config();
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(50.0),
                context_window_size: Some(0),
                ..Default::default()
            },
        };
        let output = render(&config, &input, None);
        assert!(output.is_empty());
    }

    #[test]
    fn test_full_mode_with_tokens() {
        let config = default_config();
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(25.0),
                context_window_size: Some(200_000),
                current_usage: CurrentUsage {
                    input_tokens: Some(5_000),
                    output_tokens: Some(2_000),
                    cache_read_input_tokens: Some(10_000),
                    cache_creation_input_tokens: Some(3_000),
                },
                total_input_tokens: Some(50_000),
                total_output_tokens: Some(20_000),
            },
        };
        let output = render(&config, &input, None);
        assert!(output.contains("25%"));
        assert!(output.contains("in: 5k"));
        assert!(output.contains("out: 2k"));
        assert!(output.contains("cache read: 10k"));
        assert!(output.contains("session:"));
    }

    #[test]
    fn test_compact_mode_no_tokens() {
        let mut config = default_config();
        config.mode = Mode::Compact;
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(60.0),
                context_window_size: Some(200_000),
                current_usage: CurrentUsage {
                    input_tokens: Some(5_000),
                    output_tokens: Some(2_000),
                    ..Default::default()
                },
                total_input_tokens: Some(50_000),
                total_output_tokens: Some(20_000),
            },
        };
        let output = render(&config, &input, None);
        assert!(output.contains("60%"));
        // Compact mode should not include tokens or session
        assert!(!output.contains("in: 5k"));
        assert!(!output.contains("session:"));
    }

    #[test]
    fn test_limits_with_cache() {
        let config = default_config();
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(30.0),
                context_window_size: Some(200_000),
                ..Default::default()
            },
        };
        let cache = CacheData {
            five_hour: crate::types::RateBucket {
                utilization: Some(42.0),
                resets_at: None,
            },
            seven_day: crate::types::RateBucket {
                utilization: Some(15.0),
                resets_at: None,
            },
            seven_day_sonnet: crate::types::RateBucket {
                utilization: Some(8.0),
                resets_at: None,
            },
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &input, Some(&cache));
        assert!(output.contains("5h: 42%"));
        assert!(output.contains("7d: 15%"));
        assert!(output.contains("sonnet: 8%"));
    }

    #[test]
    fn test_stale_indicator() {
        let config = default_config();
        let input = StdinData {
            context_window: ContextWindow {
                used_percentage: Some(30.0),
                context_window_size: Some(200_000),
                ..Default::default()
            },
        };
        let cache = CacheData {
            five_hour: crate::types::RateBucket {
                utilization: Some(42.0),
                resets_at: None,
            },
            seven_day: Default::default(),
            seven_day_sonnet: Default::default(),
            fetched_at: Some(0),
            is_stale: true,
        };
        let output = render(&config, &input, Some(&cache));
        assert!(output.contains("\u{26a0}"));
    }

    fn make_input() -> StdinData {
        StdinData {
            context_window: ContextWindow {
                used_percentage: Some(30.0),
                context_window_size: Some(200_000),
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_5h_timer_hidden_when_disabled() {
        let mut config = default_config();
        config.show_5h_timer = false;
        let cache = CacheData {
            five_hour: crate::types::RateBucket {
                utilization: Some(42.0),
                resets_at: Some("2099-01-01T03:52:00+00:00".to_string()),
            },
            seven_day: Default::default(),
            seven_day_sonnet: Default::default(),
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("5h: 42%"));
        assert!(!output.contains('\u{21bb}'));
    }

    #[test]
    fn test_5h_timer_shown_by_default() {
        let mut config = default_config();
        config.show_5h_timer = true;
        let cache = CacheData {
            five_hour: crate::types::RateBucket {
                utilization: Some(42.0),
                resets_at: Some("2099-01-01T03:52:00+00:00".to_string()),
            },
            seven_day: Default::default(),
            seven_day_sonnet: Default::default(),
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("5h: 42%"));
        assert!(output.contains('\u{21bb}'));
    }

    #[test]
    fn test_weekly_timer_hidden_by_default() {
        let mut config = default_config();
        config.show_weekly_timer = false;
        let cache = CacheData {
            five_hour: Default::default(),
            seven_day: crate::types::RateBucket {
                utilization: Some(15.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            seven_day_sonnet: Default::default(),
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("7d: 15%"));
        assert!(!output.contains('\u{21bb}'));
    }

    #[test]
    fn test_weekly_timer_shown_when_enabled() {
        let mut config = default_config();
        config.show_weekly_timer = true;
        let cache = CacheData {
            five_hour: Default::default(),
            seven_day: crate::types::RateBucket {
                utilization: Some(15.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            seven_day_sonnet: Default::default(),
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("7d: 15%"));
        assert!(output.contains('\u{21bb}'));
    }

    #[test]
    fn test_sonnet_timer_hidden_by_default() {
        let mut config = default_config();
        config.show_sonnet_timer = false;
        let cache = CacheData {
            five_hour: Default::default(),
            seven_day: Default::default(),
            seven_day_sonnet: crate::types::RateBucket {
                utilization: Some(8.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("sonnet: 8%"));
        assert!(!output.contains('\u{21bb}'));
    }

    #[test]
    fn test_sonnet_timer_shown_when_enabled() {
        let mut config = default_config();
        config.show_sonnet_timer = true;
        let cache = CacheData {
            five_hour: Default::default(),
            seven_day: Default::default(),
            seven_day_sonnet: crate::types::RateBucket {
                utilization: Some(8.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            fetched_at: Some(0),
            is_stale: false,
        };
        let output = render(&config, &make_input(), Some(&cache));
        assert!(output.contains("sonnet: 8%"));
        assert!(output.contains('\u{21bb}'));
    }
}
