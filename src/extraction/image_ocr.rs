//! Centralized image OCR processing.
//!
//! Provides a shared function for processing extracted images with OCR,
//! used by DOCX, PPTX, Jupyter, Markdown, and other extractors.
//!
//! # Recursion Prevention
//!
//! The OCR results produced here set `images: None` to prevent any
//! downstream consumer from triggering further image extraction on
//! OCR output. This breaks the potential cycle:
//! document → extract images → OCR images → (no further image extraction).
//!
//! # Concurrency
//!
//! Image OCR tasks are processed with a bounded concurrency limit
//! derived from the configured thread budget to prevent resource
//! exhaustion when documents contain many embedded images.

use crate::types::{ExtractedImage, ExtractionResult};

/// Process extracted images with OCR if configured.
///
/// For each image, spawns a blocking OCR task and stores the result
/// in `image.ocr_result`. If OCR is not configured or fails for an
/// individual image, that image's `ocr_result` remains `None`.
///
/// This function is the single shared implementation used by all
/// document extractors (DOCX, PPTX, Jupyter, Markdown, etc.).
///
/// # Recursion Safety
///
/// The produced `ExtractionResult` for each image explicitly sets
/// `images: None`, preventing further image extraction cycles when
/// OCR results are consumed by archive or recursive extraction paths.
///
/// # Concurrency
///
/// Concurrency is bounded by the configured thread budget
/// using a semaphore to prevent resource exhaustion.
#[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
pub async fn process_images_with_ocr(
    mut images: Vec<ExtractedImage>,
    config: &crate::core::config::ExtractionConfig,
) -> crate::Result<Vec<ExtractedImage>> {
    if images.is_empty() || config.ocr.is_none() {
        return Ok(images);
    }

    let ocr_config = config.ocr.as_ref().unwrap();
    let output_format = config.output_format.clone();

    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    // Route through the OcrBackend registry so custom backends (with triage,
    // alternate engines, etc.) participate in embedded-image OCR the same way
    // they do for PDF pages and standalone images. Falls back to the default
    // backend (normally tesseract) when nothing custom is registered.
    let backend = {
        let registry = crate::plugins::registry::get_ocr_backend_registry();
        let registry = registry.read();
        registry.get(&ocr_config.backend)?
    };

    // Attach the caller's output format to the OCR config passed to the backend.
    let mut backend_ocr_config = ocr_config.clone();
    backend_ocr_config.output_format = Some(output_format);
    let backend_ocr_config = Arc::new(backend_ocr_config);

    // Bound concurrency to prevent resource exhaustion with many images.
    let max_tasks = crate::core::config::concurrency::resolve_thread_budget(config.concurrency.as_ref());
    let semaphore = Arc::new(Semaphore::new(max_tasks));

    type OcrTaskResult = (usize, crate::Result<ExtractionResult>);
    let mut join_set: JoinSet<OcrTaskResult> = JoinSet::new();

    for (idx, image) in images.iter().enumerate() {
        let image_data = image.data.clone();
        let permit = Arc::clone(&semaphore);
        let backend_clone = Arc::clone(&backend);
        let config_clone = Arc::clone(&backend_ocr_config);

        join_set.spawn(async move {
            let _permit = permit.acquire().await.expect("semaphore should not be closed");
            let result = backend_clone.process_image(&image_data, &config_clone).await;
            (idx, result)
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        let (idx, ocr_result) = join_result.map_err(|e| crate::KreuzbergError::Ocr {
            message: format!("OCR task panicked: {}", e),
            source: None,
        })?;

        match ocr_result {
            Ok(extraction_result) => {
                // Recursion prevention: force `images: None` and keep only the
                // minimal OCR-relevant fields so downstream consumers don't
                // re-enter the embedded-image OCR path.
                let pruned = ExtractionResult {
                    content: extraction_result.content,
                    mime_type: extraction_result.mime_type,
                    ocr_elements: extraction_result.ocr_elements,
                    ..Default::default()
                };
                images[idx].ocr_result = Some(Box::new(pruned));
            }
            Err(_) => {
                images[idx].ocr_result = None;
            }
        }
    }

    Ok(images)
}
