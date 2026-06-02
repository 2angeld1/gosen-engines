use image::ImageFormat;
use std::io::Cursor;
use tracing::{info, instrument};

#[derive(Debug)]
pub enum ImageProcessorError {
    LoadError(image::ImageError),
    EncodeError(image::ImageError),
    ParsingError(String),
}

impl std::fmt::Display for ImageProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadError(e) => write!(f, "Error al cargar la imagen: {}", e),
            Self::EncodeError(e) => write!(f, "Error al codificar en WebP: {}", e),
            Self::ParsingError(s) => write!(f, "Error de procesamiento: {}", s),
        }
    }
}

impl std::error::Error for ImageProcessorError {}

/// Estructura que contiene el resultado del preprocesamiento
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    /// Bytes optimizados en formato WebP
    pub webp_bytes: Vec<u8>,
}

/// Preprocesa una imagen: calcula hashes, redimensiona y codifica en WebP
#[instrument(skip(raw_bytes))]
pub fn preprocess_image(raw_bytes: &[u8]) -> Result<ProcessedImage, ImageProcessorError> {
    info!("Iniciando preprocesamiento de imagen de {} bytes", raw_bytes.len());

    // 2. Cargar la imagen en memoria
    let dynamic_img = image::load_from_memory(raw_bytes)
        .map_err(ImageProcessorError::LoadError)?;

    // 4. Redimensionar si supera el tamaño óptimo (Max 1600px de lado largo)
    let (width, height) = (dynamic_img.width(), dynamic_img.height());
    let resized_img = if width > 1600 || height > 1600 {
        info!("Redimensionando imagen desde {}x{} a un máximo de 1600px", width, height);
        // Lanczos3 da la máxima nitidez para OCR, ideal para preservar caracteres pequeños
        dynamic_img.resize(1600, 1600, image::imageops::FilterType::Lanczos3)
    } else {
        dynamic_img
    };

    // 5. Comprimir a formato WebP
    let mut webp_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut webp_bytes);
    
    resized_img
        .write_to(&mut cursor, ImageFormat::WebP)
        .map_err(ImageProcessorError::EncodeError)?;

    info!(
        "Imagen procesada exitosamente. Bytes finales (WebP): {}. Reducción del ~{:.1}%",
        webp_bytes.len(),
        (1.0 - (webp_bytes.len() as f32 / raw_bytes.len() as f32)) * 100.0
    );

    Ok(ProcessedImage {
        webp_bytes,
    })
}
