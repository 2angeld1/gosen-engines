use serde::{Deserialize, Serialize};
use crate::ai;

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub source: String,
    pub language: String,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Suggestion {
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub line: usize,
    pub description: String,
    pub refactored_code: String,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub suggestions: Vec<Suggestion>,
    pub method: String,
}

pub async fn run(req: AnalyzeRequest) -> Result<AnalyzeResponse, String> {
    // Try Gemini first
    if let Some(key) = &req.gemini_key {
        let model = req.gemini_model.as_deref().unwrap_or("gemini-1.5-pro");
        match ai::gemini_analyze_smells(&req.source, &req.language, key, model).await {
            Ok(json_str) => {
                match serde_json::from_str::<Vec<Suggestion>>(&json_str) {
                    Ok(suggestions) => return Ok(AnalyzeResponse {
                        suggestions,
                        method: format!("gemini:{}", model),
                    }),
                    Err(e) => tracing::error!("Failed to parse gemini response: {}. Response: {}", e, json_str),
                }
            }
            Err(e) => tracing::error!("gemini analysis failed: {}", e),
        }
    }

    // Fallback to Cohere
    if let Some(key) = &req.cohere_key {
        let model = req.cohere_model.as_deref().unwrap_or("command-r-plus");
        match ai::cohere_analyze_smells(&req.source, &req.language, key, model).await {
            Ok(json_str) => {
                match serde_json::from_str::<Vec<Suggestion>>(&json_str) {
                    Ok(suggestions) => return Ok(AnalyzeResponse {
                        suggestions,
                        method: format!("cohere:{}", model),
                    }),
                    Err(e) => tracing::error!("Failed to parse cohere response: {}. Response: {}", e, json_str),
                }
            }
            Err(e) => tracing::error!("cohere analysis failed: {}", e),
        }
    }

    Err("All AI analysis backends failed or no valid API keys provided".to_string())
}
