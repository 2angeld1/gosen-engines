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

pub async fn gemini_analyze_smells(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("gemini analyze smells: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Act as a strict Tech Lead and Software Architect. Analyze the following {} code.\n\
         Find any violations of SOLID principles, DRY opportunities, and code smells.\n\
         Return ONLY a valid JSON array of objects with the following keys: \n\
         - 'type' (string: 'DRY', 'SOLID', or 'CleanCode')\n\
         - 'line' (integer: approximate line number or 0)\n\
         - 'description' (string: clear explanation)\n\
         - 'refactored_code' (string: the proposed fix)\n\
         Do NOT include markdown formatting or explanations outside the JSON array.\n\n{}",
        lang, source
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let url = format!("{}/{}:generateContent?key={}", GEMINI_BASE, model, key);
    let resp = client.post(&url).json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("gemini {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"].as_str().ok_or_else(|| "gemini: empty response".to_string())?;

    Ok(clean_code_block(text))
}

pub async fn cohere_analyze_smells(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("cohere analyze smells: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Act as a strict Tech Lead and Software Architect. Analyze the following {} code.\n\
         Find any violations of SOLID principles, DRY opportunities, and code smells.\n\
         Return ONLY a valid JSON array of objects with the following keys: \n\
         - 'type' (string: 'DRY', 'SOLID', or 'CleanCode')\n\
         - 'line' (integer: approximate line number or 0)\n\
         - 'description' (string: clear explanation)\n\
         - 'refactored_code' (string: the proposed fix)\n\
         Do NOT include markdown formatting or explanations outside the JSON array.\n\n{}",
        lang, source
    );

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let resp = client.post(COHERE_URL).header("Authorization", format!("Bearer {}", key)).header("Content-Type", "application/json").json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("cohere {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["message"]["content"][0]["text"].as_str().or_else(|| data["text"].as_str()).ok_or_else(|| "cohere: empty response".to_string())?;

    Ok(clean_code_block(text))
}

pub async fn gemini_extract_business_rules(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("gemini extract business rules: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Eres un Analista de Negocios Técnico (Business Analyst). Analiza el siguiente código en {}.\n\
         Tu objetivo es extraer las reglas de negocio puras (ej. condiciones, validaciones, flujos de decisiones lógicas) \n\
         ignorando los detalles de implementación de bajo nivel.\n\
         Devuelve la respuesta estructurada en formato Markdown, usando títulos (##), listas y negritas para resaltar las reglas clave.\n\
         La respuesta debe estar lista para ser leída por un gerente o persona no técnica.\n\n{}",
        lang, source
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let url = format!("{}/{}:generateContent?key={}", GEMINI_BASE, model, key);
    let resp = client.post(&url).json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("gemini {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"].as_str().ok_or_else(|| "gemini: empty response".to_string())?;

    Ok(text.to_string())
}

pub async fn cohere_extract_business_rules(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("cohere extract business rules: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Eres un Analista de Negocios Técnico (Business Analyst). Analiza el siguiente código en {}.\n\
         Tu objetivo es extraer las reglas de negocio puras (ej. condiciones, validaciones, flujos de decisiones lógicas) \n\
         ignorando los detalles de implementación de bajo nivel.\n\
         Devuelve la respuesta estructurada en formato Markdown, usando títulos (##), listas y negritas para resaltar las reglas clave.\n\
         La respuesta debe estar lista para ser leída por un gerente o persona no técnica.\n\n{}",
        lang, source
    );

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let resp = client.post(COHERE_URL).header("Authorization", format!("Bearer {}", key)).header("Content-Type", "application/json").json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("cohere {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["message"]["content"][0]["text"].as_str().or_else(|| data["text"].as_str()).ok_or_else(|| "cohere: empty response".to_string())?;

    Ok(text.to_string())
}

pub async fn gemini_detect_zombie(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("gemini detect zombie: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Eres un auditor estático de código experto. Analiza el siguiente código en {}.\n\
         Encuentra funciones, métodos, variables, o bloques lógicos que estén declarados pero que JAMÁS se utilicen o sean inalcanzables (Código Muerto / Zombie).\n\
         Devuelve SÓLO un JSON válido con esta estructura estricta, sin markdown extra:\n\
         {{\n  \"zombies\": [\n    {{\n      \"type\": \"FUNCIÓN_NO_USADA\",\n      \"line\": 15,\n      \"name\": \"calcular_descuento\",\n      \"reason\": \"La función está declarada pero no se invoca en ningún lugar de este archivo.\"\n    }}\n  ]\n}}\n\n{}",
        lang, source
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let url = format!("{}/{}:generateContent?key={}", GEMINI_BASE, model, key);
    let resp = client.post(&url).json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("gemini {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"].as_str().ok_or_else(|| "gemini: empty response".to_string())?;

    Ok(clean_code_block(text))
}

pub async fn cohere_detect_zombie(source: &str, lang: &str, key: &str, model: &str) -> Result<String, String> {
    tracing::debug!("cohere detect zombie: {} model={} input_len={}", lang, model, source.len());
    
    let prompt = format!(
        "Eres un auditor estático de código experto. Analiza el siguiente código en {}.\n\
         Encuentra funciones, métodos, variables, o bloques lógicos que estén declarados pero que JAMÁS se utilicen o sean inalcanzables (Código Muerto / Zombie).\n\
         Devuelve SÓLO un JSON válido con esta estructura estricta, sin markdown extra:\n\
         {{\n  \"zombies\": [\n    {{\n      \"type\": \"FUNCIÓN_NO_USADA\",\n      \"line\": 15,\n      \"name\": \"calcular_descuento\",\n      \"reason\": \"La función está declarada pero no se invoca en ningún lugar de este archivo.\"\n    }}\n  ]\n}}\n\n{}",
        lang, source
    );

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let resp = client.post(COHERE_URL).header("Authorization", format!("Bearer {}", key)).header("Content-Type", "application/json").json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("cohere {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["message"]["content"][0]["text"].as_str().or_else(|| data["text"].as_str()).ok_or_else(|| "cohere: empty response".to_string())?;

    Ok(clean_code_block(text))
}

pub async fn gemini_generate_agent_rules(source: &str, lang: &str, key: &str, model: &str, format_type: &str) -> Result<String, String> {
    tracing::debug!("gemini generate agent rules: {} model={} format={}", lang, model, format_type);
    
    let format_hint = match format_type {
        "cursorrules" => "Genera un archivo .cursorrules optimizado para Cursor AI.",
        "copilot" => "Genera un archivo .github/copilot-instructions.md optimizado para GitHub Copilot.",
        "claude" => "Genera un archivo CLAUDE.md optimizado para Claude Code / Claude AI.",
        _ => "Genera un archivo AGENTS.md genérico compatible con cualquier asistente de IA.",
    };

    let prompt = format!(
        "Eres un Ingeniero DevOps experto en documentación de proyectos para asistentes de IA.\n\
         Analiza el siguiente código en {} y genera un documento de contexto de proyecto.\n\
         {}.\n\
         El documento debe incluir:\n\
         - Descripción general del proyecto y su propósito\n\
         - Arquitectura y patrones de diseño detectados\n\
         - Convenciones de código (naming, estructura, imports)\n\
         - Dependencias principales detectadas\n\
         - Reglas de estilo y formateo\n\
         - Instrucciones para que la IA no rompa patrones existentes\n\
         Devuelve el documento en formato Markdown, listo para guardar como archivo.\n\n{}",
        lang, format_hint, source
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let url = format!("{}/{}:generateContent?key={}", GEMINI_BASE, model, key);
    let resp = client.post(&url).json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("gemini {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"].as_str().ok_or_else(|| "gemini: empty response".to_string())?;

    Ok(text.to_string())
}

pub async fn cohere_generate_agent_rules(source: &str, lang: &str, key: &str, model: &str, format_type: &str) -> Result<String, String> {
    tracing::debug!("cohere generate agent rules: {} model={} format={}", lang, model, format_type);
    
    let format_hint = match format_type {
        "cursorrules" => "Genera un archivo .cursorrules optimizado para Cursor AI.",
        "copilot" => "Genera un archivo .github/copilot-instructions.md optimizado para GitHub Copilot.",
        "claude" => "Genera un archivo CLAUDE.md optimizado para Claude Code / Claude AI.",
        _ => "Genera un archivo AGENTS.md genérico compatible con cualquier asistente de IA.",
    };

    let prompt = format!(
        "Eres un Ingeniero DevOps experto en documentación de proyectos para asistentes de IA.\n\
         Analiza el siguiente código en {} y genera un documento de contexto de proyecto.\n\
         {}.\n\
         El documento debe incluir:\n\
         - Descripción general del proyecto y su propósito\n\
         - Arquitectura y patrones de diseño detectados\n\
         - Convenciones de código (naming, estructura, imports)\n\
         - Dependencias principales detectadas\n\
         - Reglas de estilo y formateo\n\
         - Instrucciones para que la IA no rompa patrones existentes\n\
         Devuelve el documento en formato Markdown, listo para guardar como archivo.\n\n{}",
        lang, format_hint, source
    );

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| format!("reqwest: {}", e))?;
    let resp = client.post(COHERE_URL).header("Authorization", format!("Bearer {}", key)).header("Content-Type", "application/json").json(&body).send().await.map_err(|e| format!("http: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("cohere {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {}", e))?;
    let text = data["message"]["content"][0]["text"].as_str().or_else(|| data["text"].as_str()).ok_or_else(|| "cohere: empty response".to_string())?;

    Ok(text.to_string())
}
