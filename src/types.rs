use serde::Deserialize;

// -- Stdin JSON (always available) --

#[derive(Debug, Default, Deserialize)]
pub struct StdinData {
    #[serde(default)]
    pub context_window: ContextWindow,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContextWindow {
    pub used_percentage: Option<f64>,
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub current_usage: CurrentUsage,
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

// -- Rate bucket + cache (always available for render) --

#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "usage-tracking",
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct RateBucket {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

pub struct CacheData {
    pub five_hour: RateBucket,
    pub seven_day: RateBucket,
    pub seven_day_sonnet: RateBucket,
    #[allow(dead_code)]
    pub fetched_at: Option<u64>,
    pub is_stale: bool,
}

// -- API + query types (feature-gated) --

#[cfg(feature = "usage-tracking")]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct UsageApiResponse {
    pub five_hour: Option<RateBucket>,
    pub seven_day: Option<RateBucket>,
    pub seven_day_sonnet: Option<RateBucket>,
    #[serde(default)]
    pub fetched_at: Option<u64>,
}

#[cfg(feature = "usage-tracking")]
pub enum CacheReadError {
    NotFound,
    Corrupt,
    Clock,
}

#[cfg(feature = "usage-tracking")]
#[derive(serde::Serialize)]
pub struct QueryOutputFull {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_hour: Option<BucketOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day: Option<BucketOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_sonnet: Option<BucketOutput>,
    pub meta: QueryMeta,
}

#[cfg(feature = "usage-tracking")]
#[derive(serde::Serialize)]
pub struct BucketOutput {
    pub utilization: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_in: Option<String>,
}

#[cfg(feature = "usage-tracking")]
#[derive(serde::Serialize)]
pub struct QueryMeta {
    pub fetched_at: Option<u64>,
    pub is_stale: bool,
}

#[cfg(feature = "usage-tracking")]
#[derive(serde::Serialize)]
pub struct QueryOutputMinimal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_hour_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_sonnet_pct: Option<f64>,
    pub is_stale: bool,
}

// -- Credential types (feature-gated) --

#[cfg(feature = "usage-tracking")]
#[derive(Debug, Deserialize)]
pub struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: Option<OAuthData>,
}

#[cfg(feature = "usage-tracking")]
#[derive(Debug, Deserialize)]
pub struct OAuthData {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
}
