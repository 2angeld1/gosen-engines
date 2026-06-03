mod translate;
mod translate_repo;
mod ai;
mod rules;
mod detect;
mod cache;
mod languages;
mod parser;
mod db;
mod extract;
mod analyze;
mod business_rules;
mod zombie;
mod agent_rules;

use axum::{Router, routing::{post, get}, Json, extract::State, http::StatusCode};
use std::sync::Arc;
use serde::Deserialize;

struct AppState {
    db: db::DbPool,
}

#[derive(Deserialize)]
struct TranslateRequest {
    source: String,
    source_lang: String,
    target_lang: String,
    source_version: Option<String>,
    target_version: Option<String>,
    gemini_key: Option<String>,
    cohere_key: Option<String>,
    gemini_model: Option<String>,
    cohere_model: Option<String>,
}

#[derive(Deserialize)]
struct DetectRequest {
    source: String,
}

#[derive(Deserialize)]
struct TranslateRepoRequest {
    repo_url: String,
    source_lang: String,
    target_lang: String,
    branch: Option<String>,
    gemini_key: Option<String>,
    cohere_key: Option<String>,
    gemini_model: Option<String>,
    cohere_model: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_pool = db::DbPool::init().await;
    let state = Arc::new(AppState { db: db_pool });

    let app = Router::new()
        .route("/translate", post(handle_translate))
        .route("/translate-repo", post(handle_translate_repo))
        .route("/detect", post(handle_detect))
        .route("/languages", get(handle_languages))
        .route("/extract-ast", post(handle_extract))
        .route("/analyze-smells", post(handle_analyze))
        .route("/extract-business-rules", post(handle_business_rules))
        .route("/detect-zombie-code", post(handle_zombie))
        .route("/generate-agent-rules", post(handle_agent_rules))
        .route("/health", get(handle_health))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8002".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("verso-core listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handle_translate(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<translate::Response>, (StatusCode, Json<serde_json::Value>)> {
    let internal_req = translate::Request {
        source: req.source,
        source_lang: req.source_lang,
        target_lang: req.target_lang,
        source_version: req.source_version,
        target_version: req.target_version,
        gemini_key: req.gemini_key.or_else(|| std::env::var("GEMINI_API_KEY").ok()),
        cohere_key: req.cohere_key.or_else(|| std::env::var("COHERE_API_KEY").ok()),
        gemini_model: req.gemini_model,
        cohere_model: req.cohere_model,
        db: _state.db.clone(),
    };

    match translate::run(internal_req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_translate_repo(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TranslateRepoRequest>,
) -> Result<Json<translate_repo::RepoResponse>, (StatusCode, Json<serde_json::Value>)> {
    let internal_req = translate_repo::RepoRequest {
        repo_url: req.repo_url,
        source_lang: req.source_lang,
        target_lang: req.target_lang,
        branch: req.branch,
        gemini_key: req.gemini_key.or_else(|| std::env::var("GEMINI_API_KEY").ok()),
        cohere_key: req.cohere_key.or_else(|| std::env::var("COHERE_API_KEY").ok()),
        gemini_model: req.gemini_model,
        cohere_model: req.cohere_model,
        db: _state.db.clone(),
    };

    match translate_repo::run(internal_req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_detect(
    Json(req): Json<DetectRequest>,
) -> Json<serde_json::Value> {
    let lang = detect::detect_language(&req.source);
    Json(serde_json::json!({"language": lang}))
}

async fn handle_languages() -> Json<serde_json::Value> {
    Json(languages::get_all())
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "verso-core"}))
}

async fn handle_extract(
    Json(req): Json<extract::ExtractAstRequest>,
) -> Json<extract::ExtractAstResponse> {
    Json(extract::extract_ast(&req))
}

async fn handle_analyze(
    Json(mut req): Json<analyze::AnalyzeRequest>,
) -> Result<Json<analyze::AnalyzeResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.gemini_key.is_none() {
        req.gemini_key = std::env::var("GEMINI_API_KEY").ok();
    }
    if req.cohere_key.is_none() {
        req.cohere_key = std::env::var("COHERE_API_KEY").ok();
    }
    match analyze::run(req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_business_rules(
    Json(mut req): Json<business_rules::BusinessRulesRequest>,
) -> Result<Json<business_rules::BusinessRulesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.gemini_key.is_none() {
        req.gemini_key = std::env::var("GEMINI_API_KEY").ok();
    }
    if req.cohere_key.is_none() {
        req.cohere_key = std::env::var("COHERE_API_KEY").ok();
    }
    match business_rules::run(req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_zombie(
    Json(mut req): Json<zombie::ZombieRequest>,
) -> Result<Json<zombie::ZombieResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.gemini_key.is_none() {
        req.gemini_key = std::env::var("GEMINI_API_KEY").ok();
    }
    if req.cohere_key.is_none() {
        req.cohere_key = std::env::var("COHERE_API_KEY").ok();
    }
    match zombie::run(req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_agent_rules(
    Json(mut req): Json<agent_rules::AgentRulesRequest>,
) -> Result<Json<agent_rules::AgentRulesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.gemini_key.is_none() {
        req.gemini_key = std::env::var("GEMINI_API_KEY").ok();
    }
    if req.cohere_key.is_none() {
        req.cohere_key = std::env::var("COHERE_API_KEY").ok();
    }
    match agent_rules::run(req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e})),
        )),
    }
}
