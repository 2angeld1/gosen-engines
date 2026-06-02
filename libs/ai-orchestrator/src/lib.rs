use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

// Lista de modelos Gemini a competir en paralelo
pub const GEMINI_MODELS: &[&str] = &[
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash",
    "gemini-2.0-flash",
    "gemini-2.0-flash-lite",
    "gemini-flash-latest",
    "gemini-3.1-flash-lite-preview",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub success: bool,
    pub productos: Vec<Value>,
    pub fiscal: Value,
    pub total_detectados: usize,
    pub metodo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Llama a un único modelo Gemini de forma asíncrona
async fn call_single_gemini(
    client: Client,
    model: String,
    api_key: String,
    image_base64: String,
    system_prompt: String,
) -> Result<Value, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    // Estructurar el payload para la REST API oficial de Gemini
    let payload = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {
                        "inlineData": {
                            "mimeType": "image/webp",
                            "data": image_base64
                        }
                    }
                ]
            }
        ],
        "systemInstruction": {
            "parts": [
                {
                    "text": system_prompt
                }
            ]
        }
    });

    info!("Lanzando petición Gemini para modelo: {}", model);

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("HTTP request error ({}): {}", model, e))?;

    let status = response.status();
    if !status.is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("Google API returned status {} ({}): {}", status, model, err_text));
    }

    let response_json: Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parsing error ({}): {}", model, e))?;

    // Navegar en la respuesta para extraer el texto generado
    // Estructura esperada: candidates[0].content.parts[0].text
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("Invalid Gemini API response structure ({}): {:?}", model, response_json))?
        .trim();

    if text.is_empty() || text == "\"error\"" || text == "error" {
        return Err(format!("Empty or invalid model response ({}): {}", model, text));
    }

    // Limpiar bloques de código markdown si los hay
    let cleaned_text = clean_json_markdown(text);

    // Intentar deserializar como JSON válido
    let parsed_json: Value = serde_json::from_str(&cleaned_text)
        .map_err(|e| format!("Model ({}) response is not valid JSON: {}. Response text: {}", model, e, cleaned_text))?;

    // Verificar si el modelo retornó error de "no es factura"
    if parsed_json.get("error").and_then(|v| v.as_str()) == Some("not_an_invoice") {
        return Err(format!("Image is not classified as an invoice by model ({})", model));
    }

    info!("¡Modelo {} ganó la carrera exitosamente!", model);
    Ok(parsed_json)
}

/// Función auxiliar para limpiar bloques tipo ```json ... ```
fn clean_json_markdown(text: &str) -> String {
    let mut cleaned = text.trim().to_string();
    if cleaned.starts_with("```") {
        // Remover el inicio ```json o ```
        if cleaned.starts_with("```json") {
            cleaned = cleaned.replace("```json", "");
        } else {
            cleaned = cleaned.replacen("```", "", 1);
        }
        // Remover el final ```
        if cleaned.ends_with("```") {
            // Remover las últimas 3 comillas
            cleaned.truncate(cleaned.len() - 3);
        }
    }
    cleaned.trim().to_string()
}

/// Orquesta la carrera paralela entre modelos Gemini con cancelación temprana
pub async fn race_gemini_models(
    api_key: String,
    image_base64: String,
    system_prompt: String,
) -> Result<(Value, String), String> {
    let client = Client::new();
    let (tx, mut rx) = mpsc::channel(GEMINI_MODELS.len());
    let system_prompt = Arc::new(system_prompt);
    let image_base64 = Arc::new(image_base64);

    let mut handles = Vec::new();

    for model in GEMINI_MODELS {
        let tx_clone = tx.clone();
        let client_clone = client.clone();
        let model_str = model.to_string();
        let api_key_clone = api_key.clone();
        let image_clone = Arc::clone(&image_base64);
        let prompt_clone = Arc::clone(&system_prompt);

        let handle = tokio::spawn(async move {
            let result = call_single_gemini(
                client_clone,
                model_str.clone(),
                api_key_clone,
                (*image_clone).clone(),
                (*prompt_clone).clone(),
            )
            .await;

            let _ = tx_clone.send((model_str, result)).await;
        });

        handles.push(handle);
    }

    // Dropear el sender original para evitar deadlocks en el receptor
    drop(tx);

    let mut errors = Vec::new();
    let mut winner: Option<(Value, String)> = None;

    // Escuchar el primero que responda exitosamente
    while let Some((model, res)) = rx.recv().await {
        match res {
            Ok(json_val) => {
                winner = Some((json_val, model));
                break;
            }
            Err(e) => {
                warn!("Modelo {} falló en la carrera: {}", model, e);
                errors.push(format!("{}: {}", model, e));
            }
        }
    }

    // Cancelar/Abortar todas las tareas en background inmediatamente
    for handle in handles {
        handle.abort();
    }

    if let Some(data) = winner {
        Ok(data)
    } else {
        Err(format!(
            "Todos los modelos Gemini fallaron. Detalle de errores:\n{}",
            errors.join("\n")
        ))
    }
}

/// Fallback a escaneo local en Python (EasyOCR / Modo Supervivencia)
pub async fn call_emergency_ocr_fallback(
    python_service_url: &str,
    image_base64: String,
) -> Result<OrchestrationResult, String> {
    let client = Client::new();
    let url = format!("{}/agent/emergency-ocr", python_service_url);

    info!("Iniciando Fallback de Emergencia: llamando a OCR local en {}", url);

    let payload = serde_json::json!({
        "imagen": image_base64
    });

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error de red al llamar a OCR local: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Python OCR local devolvió error: {}", response.status()));
    }

    let result: Value = response
        .json()
        .await
        .map_err(|e| format!("Error parseando respuesta de OCR local: {}", e))?;

    let productos = result["productos"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let fiscal = result["fiscal"].clone();

    Ok(OrchestrationResult {
        success: true,
        productos,
        fiscal,
        total_detectados: result["total_detectados"].as_u64().unwrap_or(0) as usize,
        metodo: "blind_local_scan".to_string(),
        error: None,
    })
}
