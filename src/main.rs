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

use std::io::{self, IsTerminal, Read};

use config::Config;
use types::StdinData;

fn main() {
    #[cfg(windows)]
    enable_ansi();

    #[cfg(feature = "usage-tracking")]
    if std::env::args().any(|a| a == "--fetch") {
        fetch_mode();
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
    api::read_cache(config)
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
    unsafe {
        extern "system" {
            fn GetStdHandle(nStdHandle: u32) -> isize;
            fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
            fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
        }
        let handle = GetStdHandle(0xFFFF_FFF5); // STD_OUTPUT_HANDLE
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            let _ = SetConsoleMode(handle, mode | 0x0004); // ENABLE_VIRTUAL_TERMINAL_PROCESSING
        }
    }
}
