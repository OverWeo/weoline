mod config;
mod format;
mod render;
mod types;

#[cfg(feature = "usage-tracking")]
mod api;
#[cfg(feature = "usage-tracking")]
mod lock;
#[cfg(feature = "usage-tracking")]
mod oauth;
#[cfg(feature = "usage-tracking")]
mod query;

use std::io::{self, IsTerminal, Read};

use config::Config;
use types::StdinData;

fn main() {
    #[cfg(windows)]
    enable_ansi();

    let args: Vec<String> = std::env::args().collect();

    #[cfg(feature = "usage-tracking")]
    if args.iter().any(|a| a == "--fetch") {
        fetch_mode();
        return;
    }

    #[cfg(feature = "usage-tracking")]
    if args.iter().any(|a| a == "--query" || a == "-q") {
        query_mode(&args);
        return;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    normal_mode();
}

fn normal_mode() {
    if io::stdin().is_terminal() {
        return;
    }

    let mut buf = Vec::with_capacity(2048);
    if io::stdin().read_to_end(&mut buf).is_err() {
        return;
    }

    let input: StdinData = serde_json::from_slice(&buf).unwrap_or_default();
    let config = Config::from_env();

    let cache = get_cache(&config);

    let output = render::render(&config, &input, cache.as_ref());
    if !output.is_empty() {
        print!("{}", output);
    }
}

#[cfg(feature = "usage-tracking")]
fn query_mode(args: &[String]) {
    let qa = query::parse_query_args(args);
    let config = Config::from_env();

    if qa.refresh {
        let Some(_guard) = lock::try_acquire() else {
            eprintln!("error: another fetch is in progress");
            std::process::exit(1);
        };
        if let Err(e) = api::poll_api(&config) {
            let msg = e.to_string();
            if msg.contains("token") || msg.contains("Bearer") {
                eprintln!("error: API request failed");
            } else {
                eprintln!("error: {msg}");
            }
            std::process::exit(1);
        }
    }

    match api::read_cache(&config) {
        Ok(cache) => {
            let output = query::render_query(&cache, &qa);
            println!("{output}");
        }
        Err(types::CacheReadError::NotFound) => {
            eprintln!("error: no cache data available (run weoline in pipe mode first, or use --refresh)");
            std::process::exit(1);
        }
        Err(types::CacheReadError::Corrupt) => {
            eprintln!("error: failed to parse cache file");
            std::process::exit(1);
        }
        Err(types::CacheReadError::Clock) => {
            eprintln!("error: system clock error");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "\
weoline — Claude Code statusline

USAGE:
  weoline                              Pipe mode (stdin JSON → stdout ANSI)
  weoline --query [OPTIONS]            Query cached rate limit data

QUERY OPTIONS:
  -q, --query                          Activate query mode (no stdin)
  -f, --format <json|toon>             Output format (default: toon)
  -d, --detail <minimal|full>          Detail level (default: full)
      --filter <all|sonnet|five-hour|seven-day>
                                       Filter to specific bucket (default: all)
  -r, --refresh                        Force fresh API fetch before query
  -h, --help                           Print this help message

EXAMPLES:
  weoline --query                      Toon output, full detail, all buckets
  weoline --query -f json              JSON output, full detail, all buckets
  weoline --query -f json -d minimal   JSON, just percentages
  weoline --query -f json --filter sonnet
                                       JSON, sonnet bucket only
  weoline --query --refresh -f json    Force API refresh, then JSON output

ENVIRONMENT:
  SL_MODE              Display mode: full, compact, minimal (default: full)
  SL_SHOW_LIMITS       Show rate limits: 1/0 (default: 1)
  SL_SHOW_WEEKLY       Show 7-day limit: 1/0 (default: 1)
  SL_SHOW_SONNET       Show Sonnet limit: 1/0 (default: 1)
  SL_REFRESH_INTERVAL  Cache refresh interval in seconds (default: 300)
  SL_CACHE_FILE        Path to cache file (default: ~/.claude/usage-cache.json)"
    );
}

#[cfg(feature = "usage-tracking")]
fn get_cache(config: &Config) -> Option<types::CacheData> {
    let needs_api = !matches!(config.mode, config::Mode::Minimal) && config.show_limits;
    if !needs_api {
        return None;
    }

    if api::is_cache_stale(config) {
        if let Some(guard) = lock::try_acquire() {
            drop(guard);
            spawn_fetch();
        }
    }
    api::read_cache(config).ok()
}

#[cfg(not(feature = "usage-tracking"))]
fn get_cache(_config: &Config) -> Option<types::CacheData> {
    None
}

#[cfg(feature = "usage-tracking")]
fn fetch_mode() {
    let config = Config::from_env();
    let Some(_guard) = lock::try_acquire() else {
        return;
    };
    let _ = api::poll_api(&config);
}

#[cfg(feature = "usage-tracking")]
fn spawn_fetch() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--fetch")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(child) = cmd.spawn() {
        drop(child);
    }
}

#[cfg(windows)]
fn enable_ansi() {
    unsafe extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }
    unsafe {
        let handle = GetStdHandle(0xFFFF_FFF5); // STD_OUTPUT_HANDLE
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            let _ = SetConsoleMode(handle, mode | 0x0004); // ENABLE_VIRTUAL_TERMINAL_PROCESSING
        }
    }
}
