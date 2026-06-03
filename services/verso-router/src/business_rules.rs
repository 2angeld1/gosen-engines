use serde::{Deserialize, Serialize};
use crate::ai;

#[derive(Deserialize)]
pub struct BusinessRulesRequest {
    pub source: String,
    pub language: String,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
}

#[derive(Serialize)]
pub struct BusinessRulesResponse {
    pub markdown: String,
    pub method: String,
}

pub async fn run(req: BusinessRulesRequest) -> Result<BusinessRulesResponse, String> {
    // Try Gemini first
    if let Some(key) = &req.gemini_key {
        let model = req.gemini_model.as_deref().unwrap_or("gemini-1.5-pro");
        match ai::gemini_extract_business_rules(&req.source, &req.language, key, model).await {
            Ok(markdown) => return Ok(BusinessRulesResponse {
                markdown,
                method: format!("gemini:{}", model),
            }),
            Err(e) => tracing::error!("gemini business rules failed: {}", e),
        }
    }

    // Fallback to Cohere
    if let Some(key) = &req.cohere_key {
        let model = req.cohere_model.as_deref().unwrap_or("command-r-plus");
        match ai::cohere_extract_business_rules(&req.source, &req.language, key, model).await {
            Ok(markdown) => return Ok(BusinessRulesResponse {
                markdown,
                method: format!("cohere:{}", model),
            }),
            Err(e) => tracing::error!("cohere business rules failed: {}", e),
        }
    }

    Err("All AI backends failed or no valid API keys provided".to_string())
}
