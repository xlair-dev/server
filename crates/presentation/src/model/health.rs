use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub status: &'static str,
    pub timestamp: String,
}

impl HealthCheckResponse {
    pub fn new() -> Self {
        Self {
            status: "ok",
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for HealthCheckResponse {
    fn default() -> Self {
        Self::new()
    }
}
