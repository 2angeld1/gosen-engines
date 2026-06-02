use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Request {
    pub source: String,
    pub source_lang: String,
    pub target_lang: String,
    pub source_version: Option<String>,
    pub target_version: Option<String>,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
    pub db: crate::db::DbPool,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub result: String,
    pub lines_input: usize,
    pub lines_output: usize,
    pub method: String,
}

pub async fn run(req: Request) -> Result<Response, String> {
    let lines_input = count_lines(&req.source);
    let source_lang = if req.source_lang.is_empty() {
        let detected = crate::detect::detect_language(&req.source);
        tracing::info!(source_lang = %detected, "language auto-detected");
        detected
    } else {
        crate::languages::normalize(&req.source_lang)
    };
    let target_lang = crate::languages::normalize(&req.target_lang);

    tracing::info!(source_lang = %source_lang, target_lang = %target_lang, lines = lines_input, "translate start");

    // 1. Try cache
    if let Some(cached) = crate::cache::get(&req.source, &source_lang, &target_lang) {
        let lines_output = count_lines(&cached);
        tracing::info!("cache HIT");
        return Ok(Response {
            result: cached,
            lines_input,
            lines_output,
            method: "cache".to_string(),
        });
    }
    tracing::info!("cache MISS");

    // 2. Try AI cascade
    let mut last_error = String::new();

    // -- INICIO ORQUESTACIÓN DE RAG PARA VERSIONES --
    let mut migration_docs = String::new();
    if source_lang == target_lang {
        if let (Some(sv), Some(tv)) = (&req.source_version, &req.target_version) {
            if sv != tv {
                tracing::info!("Version migration detected: {} {} -> {}", source_lang, sv, tv);
                if let Some(docs) = req.db.get_rule(&source_lang, &target_lang, sv, tv).await {
                    tracing::info!("Migration docs found in PostgreSQL cache");
                    migration_docs = docs;
                } else {
                    tracing::info!("Migration docs not found. Calling Caitlyn Scraper...");
                    // Call Caitlyn using reqwest
                    let client = reqwest::Client::new();
                    let payload = serde_json::json!({
                        "source_lang": source_lang,
                        "target_lang": target_lang,
                        "source_version": sv,
                        "target_version": tv
                    });
                    
                    // Asumimos que Caitlyn corre en localhost:8000
                    let caitlyn_url = std::env::var("CAITLYN_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
                    
                    match client.post(&format!("{}/api/scraper/docs", caitlyn_url))
                        .json(&payload)
                        .send()
                        .await {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                    if let Some(text) = data.get("docs_text").and_then(|t| t.as_str()) {
                                        tracing::info!("Successfully scraped migration docs");
                                        migration_docs = text.to_string();
                                        // Guardar en base de datos para la próxima vez
                                        req.db.save_rule(&source_lang, &target_lang, sv, tv, text).await;
                                    }
                                }
                            } else {
                                tracing::warn!("Caitlyn scraper failed with status: {}", resp.status());
                            }
                        }
                        Err(e) => tracing::warn!("Failed to call Caitlyn scraper: {}", e),
                    }
                }
            }
        }
    }
    // -- FIN ORQUESTACIÓN DE RAG --

    if let Some(key) = &req.gemini_key {
        if !key.is_empty() {
            let model = req.gemini_model.as_deref().unwrap_or("gemini-2.0-flash");
            tracing::info!(model = %model, "trying gemini");
            match crate::ai::gemini_translate(&req.source, &source_lang, &target_lang, key, model, &migration_docs).await {
                Ok(text) => {
                    let lines_output = count_lines(&text);
                    crate::cache::set(&req.source, &source_lang, &target_lang, &text);
                    tracing::info!("gemini SUCCESS");
                    return Ok(Response {
                        result: text,
                        lines_input,
                        lines_output,
                        method: format!("gemini:{}", model),
                    });
                }
                Err(e) => {
                    tracing::warn!("gemini FAILED: {}", e);
                    last_error = format!("gemini: {}", e);
                }
            }
        }
    }

    if let Some(key) = &req.cohere_key {
        if !key.is_empty() {
            let model = req.cohere_model.as_deref().unwrap_or("command-r");
            tracing::info!(model = %model, "trying cohere");
            match crate::ai::cohere_translate(&req.source, &source_lang, &target_lang, key, model, &migration_docs).await {
                Ok(text) => {
                    let lines_output = count_lines(&text);
                    crate::cache::set(&req.source, &source_lang, &target_lang, &text);
                    tracing::info!("cohere SUCCESS");
                    return Ok(Response {
                        result: text,
                        lines_input,
                        lines_output,
                        method: format!("cohere:{}", model),
                    });
                }
                Err(e) => {
                    tracing::warn!("cohere FAILED: {}", e);
                    last_error = format!("cohere: {}", e);
                }
            }
        }
    }

    // 3. Try rules-based
    tracing::info!("trying rules");
    match crate::rules::translate(&req.source, &source_lang, &target_lang) {
        Some(text) => {
            let lines_output = count_lines(&text);
            crate::cache::set(&req.source, &source_lang, &target_lang, &text);
            tracing::info!("rules SUCCESS");
            return Ok(Response {
                result: text,
                lines_input,
                lines_output,
                method: "rules".to_string(),
            });
        }
        None => {
            tracing::warn!("rules no match");
            last_error = format!("rules: no match, last: {}", last_error);
        }
    }

    tracing::error!("translate FAILED: {}", last_error);
    Err(last_error)
}

fn count_lines(s: &str) -> usize {
    if s.is_empty() { 0 } else { s.lines().count() }
}
