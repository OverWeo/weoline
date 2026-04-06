use crate::format::{get_percent_color, RST};
use crate::types::{
    BucketOutput, CacheData, QueryMeta, QueryOutputFull, QueryOutputMinimal, RateBucket,
};

// -- Arg types --

pub struct QueryArgs {
    pub format: OutputFormat,
    pub detail: DetailLevel,
    pub filter: Filter,
    pub refresh: bool,
}

#[derive(PartialEq)]
pub enum OutputFormat {
    Json,
    Toon,
}

#[derive(PartialEq)]
pub enum DetailLevel {
    Minimal,
    Full,
}

#[derive(PartialEq)]
pub enum Filter {
    All,
    FiveHour,
    SevenDay,
    Sonnet,
}

// -- Arg parsing --

pub fn parse_query_args(args: &[String]) -> QueryArgs {
    let mut format = OutputFormat::Toon;
    let mut detail = DetailLevel::Full;
    let mut filter = Filter::All;
    let mut refresh = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "-f" => {
                if let Some(val) = args.get(i + 1) {
                    match val.as_str() {
                        "json" => format = OutputFormat::Json,
                        "toon" => format = OutputFormat::Toon,
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--detail" | "-d" => {
                if let Some(val) = args.get(i + 1) {
                    match val.as_str() {
                        "minimal" => detail = DetailLevel::Minimal,
                        "full" => detail = DetailLevel::Full,
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--filter" => {
                if let Some(val) = args.get(i + 1) {
                    match val.as_str() {
                        "all" => filter = Filter::All,
                        "sonnet" => filter = Filter::Sonnet,
                        "five-hour" => filter = Filter::FiveHour,
                        "seven-day" => filter = Filter::SevenDay,
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--refresh" | "-r" => refresh = true,
            _ => {}
        }
        i += 1;
    }

    QueryArgs {
        format,
        detail,
        filter,
        refresh,
    }
}

// -- Rendering --

pub fn render_query(cache: &CacheData, args: &QueryArgs) -> String {
    match args.format {
        OutputFormat::Json => render_json(cache, args),
        OutputFormat::Toon => render_toon(cache, args),
    }
}

fn render_json(cache: &CacheData, args: &QueryArgs) -> String {
    match args.detail {
        DetailLevel::Full => {
            let output = QueryOutputFull {
                five_hour: if include_bucket(&args.filter, &Filter::FiveHour) {
                    bucket_output(&cache.five_hour)
                } else {
                    None
                },
                seven_day: if include_bucket(&args.filter, &Filter::SevenDay) {
                    bucket_output(&cache.seven_day)
                } else {
                    None
                },
                seven_day_sonnet: if include_bucket(&args.filter, &Filter::Sonnet) {
                    bucket_output(&cache.seven_day_sonnet)
                } else {
                    None
                },
                meta: QueryMeta {
                    fetched_at: cache.fetched_at,
                    is_stale: cache.is_stale,
                },
            };
            serde_json::to_string_pretty(&output).unwrap()
        }
        DetailLevel::Minimal => {
            let output = QueryOutputMinimal {
                five_hour_pct: if include_bucket(&args.filter, &Filter::FiveHour) {
                    cache.five_hour.utilization
                } else {
                    None
                },
                seven_day_pct: if include_bucket(&args.filter, &Filter::SevenDay) {
                    cache.seven_day.utilization
                } else {
                    None
                },
                seven_day_sonnet_pct: if include_bucket(&args.filter, &Filter::Sonnet) {
                    cache.seven_day_sonnet.utilization
                } else {
                    None
                },
                is_stale: cache.is_stale,
            };
            serde_json::to_string_pretty(&output).unwrap()
        }
    }
}

fn render_toon(cache: &CacheData, args: &QueryArgs) -> String {
    let mut parts = Vec::new();

    if include_bucket(&args.filter, &Filter::FiveHour) {
        if let Some(pct) = cache.five_hour.utilization {
            let mut s = if args.detail == DetailLevel::Full {
                let color = get_percent_color(pct as u8);
                format!("{color}\u{23f1} 5h: {pct}%{RST}")
            } else {
                format!("5h: {pct}%")
            };
            if args.detail == DetailLevel::Full {
                if let Some(ref resets_at) = cache.five_hour.resets_at {
                    if let Some(cd) = countdown_plain(resets_at) {
                        s.push_str(&format!(" \u{21bb}{cd}"));
                    }
                }
            }
            parts.push(s);
        }
    }

    if include_bucket(&args.filter, &Filter::SevenDay) {
        if let Some(pct) = cache.seven_day.utilization {
            let mut s = if args.detail == DetailLevel::Full {
                let color = get_percent_color(pct as u8);
                format!("{color}\u{1f4c5} 7d: {pct}%{RST}")
            } else {
                format!("7d: {pct}%")
            };
            if args.detail == DetailLevel::Full {
                if let Some(ref resets_at) = cache.seven_day.resets_at {
                    if let Some(cd) = countdown_plain(resets_at) {
                        s.push_str(&format!(" \u{21bb}{cd}"));
                    }
                }
            }
            parts.push(s);
        }
    }

    if include_bucket(&args.filter, &Filter::Sonnet) {
        if let Some(pct) = cache.seven_day_sonnet.utilization {
            let mut s = if args.detail == DetailLevel::Full {
                let color = get_percent_color(pct as u8);
                format!("{color}\u{1f3b5} sonnet: {pct}%{RST}")
            } else {
                format!("sonnet: {pct}%")
            };
            if args.detail == DetailLevel::Full {
                if let Some(ref resets_at) = cache.seven_day_sonnet.resets_at {
                    if let Some(cd) = countdown_plain(resets_at) {
                        s.push_str(&format!(" \u{21bb}{cd}"));
                    }
                }
            }
            parts.push(s);
        }
    }

    parts.join("  ")
}

// -- Helpers --

fn include_bucket(filter: &Filter, bucket: &Filter) -> bool {
    *filter == Filter::All || *filter == *bucket
}

fn bucket_output(bucket: &RateBucket) -> Option<BucketOutput> {
    let utilization = bucket.utilization?;
    Some(BucketOutput {
        utilization,
        resets_at: bucket.resets_at.clone(),
        resets_in: bucket.resets_at.as_deref().and_then(countdown_plain),
    })
}

fn countdown_plain(resets_at: &str) -> Option<String> {
    let epoch = crate::format::iso8601_to_epoch_secs(resets_at)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff_sec = epoch - now;
    if diff_sec <= 0 {
        return Some("now".to_string());
    }
    let days = diff_sec / 86400;
    let hours = (diff_sec % 86400) / 3600;
    let mins = (diff_sec % 3600) / 60;
    if days > 0 {
        Some(format!("{days}d{hours}h"))
    } else if hours > 0 {
        Some(format!("{hours}h{mins:02}m"))
    } else {
        Some(format!("{mins}m"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RateBucket;

    fn make_cache() -> CacheData {
        CacheData {
            five_hour: RateBucket {
                utilization: Some(24.0),
                resets_at: Some("2099-01-01T03:52:00+00:00".to_string()),
            },
            seven_day: RateBucket {
                utilization: Some(8.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            seven_day_sonnet: RateBucket {
                utilization: Some(5.0),
                resets_at: Some("2099-01-04T00:00:00+00:00".to_string()),
            },
            fetched_at: Some(1743955200),
            is_stale: false,
        }
    }

    #[test]
    fn test_parse_query_args_defaults() {
        let args: Vec<String> = vec!["weoline".into(), "--query".into()];
        let qa = parse_query_args(&args);
        assert!(qa.format == OutputFormat::Toon);
        assert!(qa.detail == DetailLevel::Full);
        assert!(qa.filter == Filter::All);
        assert!(!qa.refresh);
    }

    #[test]
    fn test_parse_query_args_json_minimal_sonnet() {
        let args: Vec<String> = vec![
            "weoline".into(),
            "--query".into(),
            "--format".into(),
            "json".into(),
            "--detail".into(),
            "minimal".into(),
            "--filter".into(),
            "sonnet".into(),
        ];
        let qa = parse_query_args(&args);
        assert!(qa.format == OutputFormat::Json);
        assert!(qa.detail == DetailLevel::Minimal);
        assert!(qa.filter == Filter::Sonnet);
    }

    #[test]
    fn test_parse_query_args_short_flags() {
        let args: Vec<String> = vec![
            "weoline".into(),
            "-q".into(),
            "-f".into(),
            "json".into(),
            "-d".into(),
            "minimal".into(),
            "-r".into(),
        ];
        let qa = parse_query_args(&args);
        assert!(qa.format == OutputFormat::Json);
        assert!(qa.detail == DetailLevel::Minimal);
        assert!(qa.refresh);
    }

    #[test]
    fn test_render_json_full() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Json,
            detail: DetailLevel::Full,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["five_hour"]["utilization"], 24.0);
        assert_eq!(v["seven_day"]["utilization"], 8.0);
        assert_eq!(v["seven_day_sonnet"]["utilization"], 5.0);
        assert_eq!(v["meta"]["is_stale"], false);
    }

    #[test]
    fn test_render_json_minimal() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Json,
            detail: DetailLevel::Minimal,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["five_hour_pct"], 24.0);
        assert_eq!(v["seven_day_pct"], 8.0);
        assert_eq!(v["seven_day_sonnet_pct"], 5.0);
        assert!(v.get("meta").is_none());
    }

    #[test]
    fn test_render_json_filter_sonnet() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Json,
            detail: DetailLevel::Full,
            filter: Filter::Sonnet,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v.get("five_hour").is_none());
        assert!(v.get("seven_day").is_none());
        assert_eq!(v["seven_day_sonnet"]["utilization"], 5.0);
    }

    #[test]
    fn test_render_toon_full() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Toon,
            detail: DetailLevel::Full,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        assert!(out.contains("5h:"));
        assert!(out.contains("7d:"));
        assert!(out.contains("sonnet:"));
    }

    #[test]
    fn test_render_toon_minimal() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Toon,
            detail: DetailLevel::Minimal,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        assert!(out.contains("5h: 24%"));
        assert!(out.contains("7d: 8%"));
        assert!(out.contains("sonnet: 5%"));
        // No emoji in minimal
        assert!(!out.contains('\u{23f1}'));
        assert!(!out.contains('\u{1f4c5}'));
        assert!(!out.contains('\u{1f3b5}'));
    }

    #[test]
    fn test_countdown_plain() {
        // Far future — should produce days
        let result = countdown_plain("2099-01-04T00:00:00+00:00");
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.contains('d'), "expected days in: {s}");

        // Past — should produce "now"
        assert_eq!(countdown_plain("2020-01-01T00:00:00+00:00"), Some("now".to_string()));

        // Invalid
        assert_eq!(countdown_plain("bad"), None);
    }

    #[test]
    fn test_render_toon_full_with_all_timers() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Toon,
            detail: DetailLevel::Full,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        let arrow_count = out.matches('\u{21bb}').count();
        assert_eq!(
            arrow_count, 3,
            "expected 3 countdown arrows, got {arrow_count} in: {out}"
        );
    }

    #[test]
    fn test_render_toon_minimal_no_timers() {
        let cache = make_cache();
        let args = QueryArgs {
            format: OutputFormat::Toon,
            detail: DetailLevel::Minimal,
            filter: Filter::All,
            refresh: false,
        };
        let out = render_query(&cache, &args);
        assert!(
            !out.contains('\u{21bb}'),
            "minimal should have no countdowns: {out}"
        );
    }
}
