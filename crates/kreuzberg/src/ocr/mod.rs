//! OCR (Optical Character Recognition) subsystem.
//!
//! This module provides OCR functionality using Tesseract as the backend.
//! It includes caching, table reconstruction, hOCR parsing, and batch processing.
//!
//! # Features
//!
//! - **Tesseract integration**: Native Tesseract backend via `kreuzberg-tesseract`
//! - **Result caching**: Persistent cache for OCR results using file hashing
//! - **Table reconstruction**: Extract and reconstruct tables from hOCR/TSV output
//! - **hOCR to Markdown**: Convert hOCR format to clean Markdown
//! - **Batch processing**: Process multiple images efficiently
//! - **Language support**: Validate and configure Tesseract languages
//! - **PSM modes**: Support for all Tesseract Page Segmentation Modes
//!
//! # Example
//!
//! ```rust,no_run
//! use kreuzberg::ocr::{OcrProcessor, TesseractConfig};
//!
//! # fn example() -> Result<(), kreuzberg::ocr::error::OcrError> {
//! let processor = OcrProcessor::new(None)?;
//! let config = TesseractConfig::default();
//!
//! let image_bytes = std::fs::read("scanned.png").expect("failed to read image");
//! let result = processor.process_image(&image_bytes, &config)?;
//!
//! println!("Extracted text: {}", result.content);
//! # Ok(())
//! # }
//! ```
//!
//! # Optional Feature
//!
//! This module requires the `ocr` feature to be enabled:
//! ```toml
//! [dependencies]
//! kreuzberg = { version = "4.0", features = ["ocr"] }
//! ```
mod backends;
#[cfg(feature = "ocr")]
pub mod cache;
#[cfg(any(feature = "ocr", feature = "paddle-ocr"))]
pub mod conversion;
pub mod error;
#[cfg(feature = "ocr")]
pub mod hocr_parser;
pub mod language_registry;
#[cfg(feature = "layout-detection")]
pub mod layout_assembly;
#[cfg(feature = "ocr")]
pub mod processor;
#[cfg(feature = "ocr")]
pub mod table;
#[cfg(feature = "ocr")]
pub mod tessdata_manager;
#[cfg(feature = "ocr")]
pub mod tesseract_backend;
#[cfg(all(feature = "ocr-wasm", not(feature = "ocr")))]
pub mod tesseract_wasm_backend;
pub mod types;
#[cfg(feature = "ocr")]
pub mod utils;
#[cfg(feature = "ocr")]
pub mod validation;

#[cfg(feature = "ocr")]
pub use cache::{OcrCache, OcrCacheStats};
pub use error::OcrError;
pub use language_registry::LanguageRegistry;
#[cfg(feature = "ocr")]
pub use processor::OcrProcessor;
#[cfg(feature = "ocr")]
pub use tessdata_manager::TessdataManager;
#[cfg(feature = "ocr")]
pub use tesseract_backend::TesseractBackend;
#[cfg(all(feature = "ocr-wasm", not(feature = "ocr")))]
pub use tesseract_wasm_backend::TesseractWasmBackend;
pub use types::{BatchItemResult, ExtractionResult, PSMMode, Table, TesseractConfig};
#[cfg(feature = "ocr")]
pub use utils::compute_hash;
