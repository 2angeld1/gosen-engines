use serde::{Deserialize, Serialize};
use crate::ai;

#[derive(Deserialize)]
pub struct ZombieRequest {
    pub source: String,
    pub language: String,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ZombieEntry {
    #[serde(rename = "type")]
    pub zombie_type: String,
    pub line: u32,
    pub name: String,
    pub reason: String,
}

#[derive(Serialize)]
pub struct ZombieResponse {
    pub zombies: Vec<ZombieEntry>,
    pub method: String,
}

pub async fn run(req: ZombieRequest) -> Result<ZombieResponse, String> {
    // Try Gemini first
    if let Some(key) = &req.gemini_key {
        let model = req.gemini_model.as_deref().unwrap_or("gemini-1.5-pro");
        match ai::gemini_detect_zombie(&req.source, &req.language, key, model).await {
            Ok(raw) => {
                if let Ok(parsed) = parse_zombie_json(&raw) {
                    return Ok(ZombieResponse {
                        zombies: parsed,
                        method: format!("gemini:{}", model),
                    });
                }
                tracing::warn!("gemini zombie: failed to parse JSON response");
            }
            Err(e) => tracing::error!("gemini zombie failed: {}", e),
        }
    }

    // Fallback to Cohere
    if let Some(key) = &req.cohere_key {
        let model = req.cohere_model.as_deref().unwrap_or("command-r-plus");
        match ai::cohere_detect_zombie(&req.source, &req.language, key, model).await {
            Ok(raw) => {
                if let Ok(parsed) = parse_zombie_json(&raw) {
                    return Ok(ZombieResponse {
                        zombies: parsed,
                        method: format!("cohere:{}", model),
                    });
                }
                tracing::warn!("cohere zombie: failed to parse JSON response");
            }
            Err(e) => tracing::error!("cohere zombie failed: {}", e),
        }
    }

    Err("All AI backends failed or no valid API keys provided".to_string())
}

fn parse_zombie_json(raw: &str) -> Result<Vec<ZombieEntry>, String> {
    // Try to parse the whole response as an object with "zombies" key
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(arr) = val.get("zombies").and_then(|v| v.as_array()) {
            let mut entries = Vec::new();
            for item in arr {
                let zombie_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("UNKNOWN").to_string();
                let line = item.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let reason = item.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
                entries.push(ZombieEntry { zombie_type, line, name, reason });
            }
            return Ok(entries);
        }
    }
    Err("Could not parse zombie JSON".to_string())
}
