use axum::{
    extract::{State, DefaultBodyLimit},
    http::{StatusCode, HeaderMap},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use base64::{engine::general_purpose, Engine as _};

// Importar nuestras librerías internas
use ai_orchestrator::{call_emergency_ocr_fallback, race_gemini_models, OrchestrationResult};
use image_processor::preprocess_image;

#[derive(Clone)]
struct AppState {
    caitlyn_url: String,
    gemini_api_key: String,
}

#[derive(Deserialize)]
struct InvoiceRequest {
    imagen: String,
    negocio_tipo: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializar logs de Tracing (Premium)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,kitchy_router=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Iniciando Caitlyn Rust Router (gosen-engines)...");

    // 2. Cargar variables de entorno
    let gemini_api_key = std::env::var("GEMINI_API_KEY")
        .unwrap_or_default();
    let caitlyn_url = std::env::var("CAITLYN_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // 4. Compartir estado de la aplicación
    let state = AppState {
        caitlyn_url,
        gemini_api_key,
    };

    // 5. Definir rutas Axum
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/agent/invoice", post(process_invoice_handler))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024)) // Límite de 20MB para imágenes base64 grandes
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 6. Arrancar el servidor
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Servidor Axum escuchando en: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

/// Handler principal para el procesamiento de facturas
async fn process_invoice_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<InvoiceRequest>,
) -> Result<Json<OrchestrationResult>, (StatusCode, Json<Value>)> {
    let negocio_tipo = payload.negocio_tipo.unwrap_or_else(|| "GASTRONOMIA".to_string());

    // 0. Extraer la API Key de Gemini desde la cabecera, con fallback al AppState
    let gemini_api_key = headers
        .get("x-gemini-key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| state.gemini_api_key.clone());

    if gemini_api_key.trim().is_empty() {
        error!("Error: GEMINI_API_KEY no provista en las variables de entorno ni en la cabecera x-gemini-key");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Falta configurar la clave de API de Gemini. Envíala en la cabecera HTTP 'x-gemini-key' o configúrala en el entorno de Rust."
            })),
        ));
    }

    // 1. Decodificar la imagen base64 original
    let raw_image_bytes = match decode_base64(&payload.imagen) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Error decodificando imagen base64: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "success": false, "error": e })),
            ));
        }
    };

    // 2. Preprocesar la imagen (Resize, WebP, Hashes)
    let processed = match preprocess_image(&raw_image_bytes) {
        Ok(img) => img,
        Err(e) => {
            error!("Error en preprocesamiento de imagen: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "success": false, "error": e.to_string() })),
            ));
        }
    };

    // 3. INFERENCIA CON MODELOS (Carrera Paralela)
    
    // Preparar el base64 de la imagen redimensionada y comprimida en WebP para Gemini
    let webp_base64 = general_purpose::STANDARD.encode(&processed.webp_bytes);

    // Prompt del sistema estructurado de forma idéntica al original de Python
    let system_prompt = format!(
        "Eres Caitlyn, un asistente de contabilidad e inventario versátil para negocios en Panamá. \
        Recibirás una imagen de una factura y el tipo de negocio ({negocio_tipo}). \
        PRIMERA REGLA: Verifica si la imagen es realmente una factura, recibo o ticket de compra. \
        Si NO es una factura, responde EXACTAMENTE: \
        {{\"error\": \"not_an_invoice\"}}\n\n\
        Si SÍ es una factura, extrae la información fiscal y TODOS los productos listados. \
        Responde ÚNICAMENTE con un objeto JSON válido, sin texto adicional, sin markdown. \
        La estructura debe ser exactamente esta:\n\
        {{\n\
          \"fiscal\": {{\n\
            \"proveedor\": \"string\", \"ruc\": \"string\", \"dv\": \"string\", \"nroFactura\": \"string\", \
            \"fecha\": \"ISO string\", \"receptor\": \"string\", \"subtotal\": number, \"itbms\": number, \"total\": number\n\
          }},\n\
          \"productos\": [\n\
            {{\n\
              \"nombre\": \"string\", \n\
              \"cantidad\": number, \n\
              \"unidad\": \"string\", \n\
              \"unidadesPorEmpaque\": number, \n\
              \"precioUnitario\": number, \n\
              \"categoriaSugerida\": \"string (insumo | reventa | ingrediente | limpieza)\", \n\
              \"precioReventaSugerido\": number | null\n\
            }}\n\
          ]\n\
        }}\n\n\
        REGLAS ESPECÍFICAS PARA {negocio_tipo}:\n\
        - Si el tipo es 'BELLEZA':\n\
          * Identifica si el producto es un 'insumo' (ej: tinte, agua oxigenada, cera) o para 'reventa' (ej: shampoo 250ml, cremas de peinar).\n\
          * Si es 'reventa', sugiere un 'precioReventaSugerido' aplicando un margen del 65% sobre el precio unitario final.\n\
          * Si es 'insumo', usa null en 'precioReventaSugerido'.\n\
          * Las categorías sugeridas deben ser 'insumo' o 'reventa'.\n\
        - Si el tipo es 'GASTRONOMIA' (default):\n\
          * Las categorías deben ser 'ingrediente', 'bebida', 'postre' o 'limpieza'.\n\
          * 'precioReventaSugerido' suele ser null a menos que sea un producto de reventa directa (ej: una soda de lata).\n\n\
        Reglas Generales:\n\
        - Si el nombre indica pack (ej: 'Sodas x 12'), extrae el número en 'unidadesPorEmpaque'.\n\
        - Si no puedes leer algún campo, usa null.\n\
        - Responde en español con tildes reales (UTF-8).",
        negocio_tipo = negocio_tipo
    );

    info!("Falla de caché. Iniciando carrera de modelos Gemini...");

    // Ejecutar carrera de modelos paralelos
    let gemini_result = race_gemini_models(
        gemini_api_key,
        webp_base64.clone(),
        system_prompt,
    )
    .await;

    match gemini_result {
        Ok((json_val, model_winner)) => {
            info!("🏁 Modelo '{}' ganó la carrera. Guardando en caché...", model_winner);

            let productos = json_val["productos"].as_array().cloned().unwrap_or_default();
            let fiscal = json_val["fiscal"].clone();
            let total_detectados = productos.len();

            let response_obj = OrchestrationResult {
                success: true,
                productos,
                fiscal,
                total_detectados,
                metodo: format!("gemini_{}", model_winner),
                error: None,
            };

            Ok(Json(response_obj))
        }
        Err(gemini_errors) => {
            warn!("⚠️ Carrera falló por completo: {}. Activando Fallback OCR local...", gemini_errors);

            // 5. MODO SUPERVIVENCIA (EasyOCR local en Python)
            // Pasamos la imagen comprimida WebP ya optimizada para reducir ancho de banda
            match call_emergency_ocr_fallback(&state.caitlyn_url, webp_base64).await {
                Ok(fallback_res) => {
                    info!("✅ Fallback a OCR local completado con éxito.");
                    Ok(Json(fallback_res))
                }
                Err(fallback_err) => {
                    error!("💥 Fallback de emergencia falló: {}", fallback_err);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Falla total en el procesamiento. Gemini error: {}. Local OCR error: {}", gemini_errors, fallback_err)
                        })),
                    ))
                }
            }
        }
    }
}

/// Helper para decodificar base64 manejando prefijos de URI de datos
fn decode_base64(encoded: &str) -> Result<Vec<u8>, &'static str> {
    let clean_str = if encoded.starts_with("data:") {
        match encoded.split_once(',') {
            Some((_, base64_part)) => base64_part,
            None => return Err("Formato URI de datos base64 inválido"),
        }
    } else {
        encoded
    };

    general_purpose::STANDARD.decode(clean_str).map_err(|_| "Error al decodificar string base64")
}
