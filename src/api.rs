use std::error::Error;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Config, STALE_MULTIPLIER};
use crate::types::{CacheData, UsageApiResponse};

const API_URL: &str = "https://api.anthropic.com/api/oauth/usage";

pub fn poll_api(config: &Config) -> Result<(), Box<dyn Error>> {
    let token = crate::oauth::get_oauth_token(&config.credentials_file)
        .ok_or("no OAuth token available")?;

    let agent = build_agent();
    let mut resp = agent
        .get(API_URL)
        .header("Authorization", &format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .call()
        .map_err(|e| format!("API request failed: {e}"))?;

    let body = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("failed to read response: {e}"))?;

    let mut response: UsageApiResponse = serde_json::from_str(&body)?;

    if response.five_hour.is_none() {
        return Err("API response missing five_hour data".into());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    response.fetched_at = Some(now);

    // Atomic write via temp file + rename
    let cache_dir = config
        .cache_file
        .parent()
        .ok_or("cache file has no parent directory")?;
    fs::create_dir_all(cache_dir)?;

    let tmp_path = cache_dir.join(".usage-cache.tmp");
    let data = serde_json::to_string(&response)?;

    if let Err(e) = fs::write(&tmp_path, &data) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e.into());
    }
    if let Err(e) = fs::rename(&tmp_path, &config.cache_file) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e.into());
    }

    Ok(())
}

pub fn read_cache(config: &Config) -> Option<CacheData> {
    let data = fs::read(&config.cache_file).ok()?;
    let resp: UsageApiResponse = serde_json::from_slice(&data).ok()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    let fetched_at = resp.fetched_at;
    let age = now.saturating_sub(fetched_at.unwrap_or(0));
    let is_stale = age > config.refresh_interval * STALE_MULTIPLIER;

    Some(CacheData {
        five_hour: resp.five_hour.unwrap_or_default(),
        seven_day: resp.seven_day.unwrap_or_default(),
        seven_day_sonnet: resp.seven_day_sonnet.unwrap_or_default(),
        fetched_at,
        is_stale,
    })
}

pub fn is_cache_stale(config: &Config) -> bool {
    let Ok(data) = fs::read(&config.cache_file) else {
        return true;
    };
    let Ok(resp) = serde_json::from_slice::<UsageApiResponse>(&data) else {
        return true;
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let age = now.saturating_sub(resp.fetched_at.unwrap_or(0));
    age >= config.refresh_interval
}

fn build_agent() -> ureq::Agent {
    use ureq::tls::{TlsConfig, TlsProvider};

    let tls_provider = {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        { TlsProvider::NativeTls }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        { TlsProvider::Rustls }
    };

    ureq::Agent::config_builder()
        .tls_config(TlsConfig::builder().provider(tls_provider).build())
        .timeout_global(Some(std::time::Duration::from_secs(5)))
        .build()
        .new_agent()
}
