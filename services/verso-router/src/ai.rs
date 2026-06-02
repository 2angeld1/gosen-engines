use serde_json::json;

const GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const COHERE_URL: &str = "https://api.cohere.com/v2/chat";

pub async fn gemini_translate(source: &str, source_lang: &str, target_lang: &str, key: &str, model: &str, migration_docs: &str) -> Result<String, String> {
    tracing::debug!("gemini request: {} -> {} model={} input_len={}", source_lang, target_lang, model, source.len());
    
    let docs_context = if !migration_docs.is_empty() {
        format!("\n\nCRITICAL OFFICIAL MIGRATION RULES TO FOLLOW:\n{}", migration_docs)
    } else {
        String::new()
    };

    let prompt = format!(
        "Translate the following code from {} to {}.\n\
         Return ONLY the translated code, no explanations, no markdown formatting.\n\
         Keep the same functionality, logic, and comments.{}\n\n{}",
        source_lang, target_lang, docs_context, source
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("reqwest: {}", e))?;

    let url = format!("{}/{}:generateContent?key={}", GEMINI_BASE, model, key);
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("gemini {}: {}", status, text));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;

    let text = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| "gemini: empty response".to_string())?;

    Ok(clean_code_block(text))
}

pub async fn cohere_translate(source: &str, source_lang: &str, target_lang: &str, key: &str, model: &str, migration_docs: &str) -> Result<String, String> {
    tracing::debug!("cohere request: {} -> {} model={} input_len={}", source_lang, target_lang, model, source.len());
    
    let docs_context = if !migration_docs.is_empty() {
        format!("\n\nCRITICAL OFFICIAL MIGRATION RULES TO FOLLOW:\n{}", migration_docs)
    } else {
        String::new()
    };

    let prompt = format!(
        "Translate the following code from {} to {}.\n\
         Return ONLY the translated code, no explanations, no markdown formatting.\n\
         Keep the same functionality, logic, and comments.{}\n\n{}",
        source_lang, target_lang, docs_context, source
    );

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("reqwest: {}", e))?;

    let resp = client
        .post(COHERE_URL)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("cohere {}: {}", status, text));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;

    let text = data["message"]["content"][0]["text"]
        .as_str()
        .or_else(|| data["text"].as_str())
        .ok_or_else(|| "cohere: empty response".to_string())?;

    Ok(clean_code_block(text))
}

fn clean_code_block(text: &str) -> String {
    let t = text.trim();
    // Remove markdown code fences
    if t.starts_with("```") {
        let without_fence = t.trim_start_matches(|c| c == '`' || c == '\n');
        let mut lines = without_fence.lines();
        // Skip language tag line if present
        let first = lines.next().unwrap_or("");
        if first.contains('\n') || first.is_empty() || first.contains(' ') || first.contains(';') || first.contains('{') {
            // First line is actual code, include it
            let rest: Vec<&str> = without_fence.lines().collect();
            let end = rest.iter().rposition(|l| !l.trim().is_empty()).unwrap_or(rest.len().saturating_sub(1));
            let code: Vec<&str> = rest[..=end].into_iter().filter(|l| !l.trim_end().ends_with("```")).copied().collect();
            return code.join("\n").trim().to_string();
        }
        // First line was the language tag, skip it
        let rest: Vec<&str> = without_fence.lines().skip(1).collect();
        let end = rest.iter().rposition(|l| !l.trim().is_empty()).unwrap_or(rest.len().saturating_sub(1));
        let code: Vec<&str> = rest[..=end].into_iter().filter(|l| !l.trim_end().ends_with("```")).copied().collect();
        code.join("\n").trim().to_string()
    } else {
        t.to_string()
    }
}
