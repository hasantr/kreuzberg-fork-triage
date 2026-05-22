//! PDF page rendering using pdf_oxide.

use crate::Result;
use crate::error::KreuzbergError;

/// Render a single PDF page to PNG bytes.
///
/// Returns raw PNG-encoded bytes for the specified page at the given DPI.
/// Uses pdf_oxide with tiny-skia for pure-Rust rendering.
///
/// # Arguments
///
/// * `pdf_bytes` - Raw PDF file bytes
/// * `page_index` - Zero-based page index
/// * `dpi` - Resolution in dots per inch (default: 150)
/// * `password` - Optional password for encrypted PDFs
///
/// # Errors
///
/// Returns `KreuzbergError::Parsing` if the PDF cannot be opened, authenticated,
/// or rendered, or if `page_index` is out of range.
pub fn render_pdf_page_to_png(
    pdf_bytes: &[u8],
    page_index: usize,
    dpi: Option<i32>,
    password: Option<&str>,
) -> Result<Vec<u8>> {
    let doc = pdf_oxide::PdfDocument::from_bytes(pdf_bytes.to_vec()).map_err(|e| KreuzbergError::Parsing {
        message: format!("Failed to open PDF: {e}"),
        source: None,
    })?;

    if let Some(pwd) = password {
        doc.authenticate(pwd.as_bytes()).map_err(|e| KreuzbergError::Parsing {
            message: format!("Failed to authenticate PDF: {e}"),
            source: None,
        })?;
    }

    let page_count = doc.page_count().map_err(|e| KreuzbergError::Parsing {
        message: format!("Failed to read page count: {e}"),
        source: None,
    })?;

    if page_index >= page_count {
        return Err(KreuzbergError::Parsing {
            message: format!("Page index {page_index} out of range (document has {page_count} pages)"),
            source: None,
        });
    }

    let render_dpi = dpi.unwrap_or(150).max(1) as u32;
    let options = pdf_oxide::rendering::RenderOptions::with_dpi(render_dpi);
    let rendered =
        pdf_oxide::rendering::render_page(&doc, page_index, &options).map_err(|e| KreuzbergError::Parsing {
            message: format!("Failed to render page {page_index}: {e}"),
            source: None,
        })?;

    Ok(rendered.data)
}

/// Geometry + pixels returned by [`render_pdf_page_to_rgba`].
///
/// `pixels` is premultiplied RGBA8 (alpha pre-applied to RGB channels),
/// row-major, top-left origin. `pixels.len() == (width * height * 4) as usize`.
#[derive(Debug, Clone)]
pub struct RenderedPagePixels {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Render a single PDF page to a raw premultiplied RGBA8 pixel buffer.
///
/// Hot-path companion to [`render_pdf_page_to_png`]. Skips PNG
/// encoding entirely — `pdf_oxide`'s `RawRgba8` output format hands
/// over pixels straight from `tiny-skia`'s pixmap. Callers that feed
/// the result into OCR triage / a raw-input OCR backend save the
/// ~10-30 ms PNG encode plus the matching decode on the consumer side.
///
/// # Arguments
///
/// Same as [`render_pdf_page_to_png`].
///
/// # Errors
///
/// Same as [`render_pdf_page_to_png`], plus an internal consistency
/// error if `pdf_oxide` returns a buffer whose length doesn't match
/// `width * height * 4` (should never happen with the `RawRgba8`
/// format, but checked because a downstream mis-interpretation would
/// be a silent memory-safety hazard).
pub fn render_pdf_page_to_rgba(
    pdf_bytes: &[u8],
    page_index: usize,
    dpi: Option<i32>,
    password: Option<&str>,
) -> Result<RenderedPagePixels> {
    let doc = pdf_oxide::PdfDocument::from_bytes(pdf_bytes.to_vec()).map_err(|e| KreuzbergError::Parsing {
        message: format!("Failed to open PDF: {e}"),
        source: None,
    })?;

    if let Some(pwd) = password {
        doc.authenticate(pwd.as_bytes()).map_err(|e| KreuzbergError::Parsing {
            message: format!("Failed to authenticate PDF: {e}"),
            source: None,
        })?;
    }

    let page_count = doc.page_count().map_err(|e| KreuzbergError::Parsing {
        message: format!("Failed to read page count: {e}"),
        source: None,
    })?;

    if page_index >= page_count {
        return Err(KreuzbergError::Parsing {
            message: format!("Page index {page_index} out of range (document has {page_count} pages)"),
            source: None,
        });
    }

    let render_dpi = dpi.unwrap_or(150).max(1) as u32;
    let options = pdf_oxide::rendering::RenderOptions::with_dpi(render_dpi).as_raw();
    let rendered =
        pdf_oxide::rendering::render_page(&doc, page_index, &options).map_err(|e| KreuzbergError::Parsing {
            message: format!("Failed to render page {page_index}: {e}"),
            source: None,
        })?;

    let expected = (rendered.width as usize)
        .checked_mul(rendered.height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| KreuzbergError::Parsing {
            message: format!(
                "PDF page {page_index} render geometry overflow: {}x{}",
                rendered.width, rendered.height
            ),
            source: None,
        })?;
    if rendered.data.len() != expected {
        return Err(KreuzbergError::Parsing {
            message: format!(
                "PDF page {page_index} raw render produced {} bytes, expected {} ({}x{}x4)",
                rendered.data.len(),
                expected,
                rendered.width,
                rendered.height
            ),
            source: None,
        });
    }

    Ok(RenderedPagePixels {
        pixels: rendered.data,
        width: rendered.width,
        height: rendered.height,
    })
}
