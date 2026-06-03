use serde::{Deserialize, Serialize};
use crate::ai;

#[derive(Deserialize)]
pub struct AgentRulesRequest {
    pub source: String,
    pub language: String,
    /// Target format: "agents" (generic), "cursorrules", "copilot", "claude"
    pub format: Option<String>,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
}

#[derive(Serialize)]
pub struct AgentRulesResponse {
    pub markdown: String,
    pub format: String,
    pub suggested_filename: String,
    pub method: String,
}

fn get_suggested_filename(format: &str) -> String {
    match format {
        "cursorrules" => ".cursorrules".to_string(),
        "copilot" => ".github/copilot-instructions.md".to_string(),
        "claude" => "CLAUDE.md".to_string(),
        _ => "AGENTS.md".to_string(),
    }
}

pub async fn run(req: AgentRulesRequest) -> Result<AgentRulesResponse, String> {
    let format = req.format.as_deref().unwrap_or("agents");

    // Try Gemini first
    if let Some(key) = &req.gemini_key {
        let model = req.gemini_model.as_deref().unwrap_or("gemini-1.5-pro");
        match ai::gemini_generate_agent_rules(&req.source, &req.language, key, model, format).await {
            Ok(markdown) => return Ok(AgentRulesResponse {
                markdown,
                format: format.to_string(),
                suggested_filename: get_suggested_filename(format),
                method: format!("gemini:{}", model),
            }),
            Err(e) => tracing::error!("gemini agent rules failed: {}", e),
        }
    }

    // Fallback to Cohere
    if let Some(key) = &req.cohere_key {
        let model = req.cohere_model.as_deref().unwrap_or("command-r-plus");
        match ai::cohere_generate_agent_rules(&req.source, &req.language, key, model, format).await {
            Ok(markdown) => return Ok(AgentRulesResponse {
                markdown,
                format: format.to_string(),
                suggested_filename: get_suggested_filename(format),
                method: format!("cohere:{}", model),
            }),
            Err(e) => tracing::error!("cohere agent rules failed: {}", e),
        }
    }

    Err("All AI backends failed or no valid API keys provided".to_string())
}
