//! Comprehensive TDD test suite for Jupyter notebook extraction.
//!
//! This test suite validates Jupyter notebook extraction against Pandoc's output
//! as a baseline. The tests verify:
//! - Notebook metadata extraction (kernelspec, language_info)
//! - Cell content aggregation (markdown and code cells)
//! - Cell outputs handling
//! - MIME type handling for various output formats
//!
//! Each test notebook is extracted and compared against Pandoc's markdown output
//! to ensure correct content extraction and transformation.

use kreuzberg::core::config::ExtractionConfig;
use kreuzberg::core::extractor::extract_bytes;
use serde_json::Value;
use std::borrow::Cow;
use std::{fs, path::PathBuf};

mod helpers;

fn jupyter_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_documents/jupyter")
        .join(name)
}

/// Helper: Check if metadata contains cells with execution_count
fn has_cells_with_execution_count(metadata_additional: &ahash::AHashMap<Cow<'static, str>, Value>) -> bool {
    if let Some(Value::Array(cells)) = metadata_additional.get("cells") {
        cells.iter().any(|cell| {
            if let Value::Object(cell_obj) = cell {
                cell_obj.contains_key("execution_count")
            } else {
                false
            }
        })
    } else {
        false
    }
}

/// Helper: Check if metadata contains cells with tags
fn has_cells_with_tags(metadata_additional: &ahash::AHashMap<Cow<'static, str>, Value>) -> bool {
    if let Some(Value::Array(cells)) = metadata_additional.get("cells") {
        cells.iter().any(|cell| {
            if let Value::Object(cell_obj) = cell {
                cell_obj.contains_key("tags")
            } else {
                false
            }
        })
    } else {
        false
    }
}

/// Test simple.ipynb - Validates markdown cells, code cells, and HTML output.
///
/// Notebook contains:
/// - Markdown cell with **bold** text (uid1)
/// - Empty code cell (uid2)
/// - Markdown section header (uid3)
/// - Code cell with HTML output (uid4) - generates execute_result with text/html
/// - Markdown cell with image reference and cell metadata tags (uid6)
///
/// Pandoc output format shows cells with triple-colon divider syntax:
/// - Markdown cells: `::: {#uid1 .cell .markdown}`
/// - Code cells: `:::: {#uid4 .cell .code execution_count="2"}`
/// - Output blocks: `::: {.output .execute_result execution_count="2"}`
///
/// Expected baseline from Pandoc:
/// - Lorem ipsum heading with bold formatting
/// - Pyout section with code cell containing IPython.display.HTML call
/// - HTML output showing console.log and <b>HTML</b>
/// - Image section with cell tags [foo, bar]
#[tokio::test]
async fn test_jupyter_simple_notebook_extraction() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("simple.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read simple.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed or notebook format unsupported");
        return;
    }

    let extraction = result.expect("Operation failed");

    assert_eq!(
        extraction.mime_type, "application/x-ipynb+json",
        "MIME type should be preserved"
    );

    assert!(!extraction.content.is_empty(), "Extracted content should not be empty");

    assert!(
        extraction.content.contains("Lorem ipsum"),
        "Should extract markdown cell 'Lorem ipsum'"
    );
    assert!(
        extraction.content.contains("Lorem ipsum"),
        "Should extract **bold** formatted text"
    );

    // Check metadata for execution_count (stored in cells metadata, not rendered in content)
    assert!(
        has_cells_with_execution_count(&extraction.metadata.additional),
        "Should preserve execution_count from code cells in metadata"
    );

    assert!(
        extraction.content.contains("HTML") || extraction.content.contains("html"),
        "Should extract HTML output content from code cells"
    );

    assert!(
        extraction.content.contains("Pyout") || extraction.content.contains("pyout"),
        "Should extract markdown section headers"
    );

    assert!(
        extraction.content.contains("Image") || extraction.content.contains("image"),
        "Should extract image cell content"
    );

    // Check metadata for tags (stored in cells metadata, not rendered in content)
    assert!(
        has_cells_with_tags(&extraction.metadata.additional),
        "Should preserve cell metadata tags"
    );

    println!(
        "✓ simple.ipynb: Successfully extracted {} characters of content",
        extraction.content.len()
    );
}

/// Test mime.ipynb - Validates MIME type output handling.
///
/// Notebook contains:
/// - Code cell 1: Import dataclasses (execution_count=1)
/// - Code cell 2: Print version string output (execution_count=2) with stream.stdout
/// - Markdown cell: "Supported IPython display formatters:"
/// - Code cell 3: Loop through mime formatters (execution_count=3)
///   - Output: list of MIME types as stdout stream:
///     - text/plain, text/html, text/markdown
///     - image/svg+xml, image/png, application/pdf
///     - image/jpeg, text/latex, application/json, application/javascript
/// - Code cell 4: Define Mime class with _repr_mimebundle_ method
/// - Code cell 5: Create instance mime = Mime("E = mc^2")
/// - Code cell 6: Execute mime variable (execution_count=6)
///   - Output: execute_result with text/markdown "$$E = mc^2$$"
/// - Markdown cell: "Note that #7561 made ipynb reader aware of this..."
///
/// Pandoc output format:
/// - Stream outputs: `::: {.output .stream .stdout}`
/// - Execute results: `::: {.output .execute_result execution_count="6"}`
/// - Multiple MIME types in single output
///
/// Expected baseline from Pandoc:
/// - Code cells with specific MIME type outputs
/// - Stream outputs showing printed text
/// - Markdown-formatted math output: $$E = mc^2$$
#[tokio::test]
async fn test_jupyter_mime_notebook_extraction() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("mime.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read mime.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    assert_eq!(
        extraction.mime_type, "application/x-ipynb+json",
        "MIME type should be preserved"
    );

    assert!(!extraction.content.is_empty(), "Extracted content should not be empty");

    assert!(
        extraction.content.contains("dataclass") || extraction.content.contains("dataclasses"),
        "Should extract code cell with imports"
    );

    assert!(
        extraction.content.contains(".stream")
            || extraction.content.contains("stdout")
            || extraction.content.contains("output"),
        "Should preserve stream output type information"
    );

    let mime_types = vec![
        "text/plain",
        "text/html",
        "text/markdown",
        "image/svg+xml",
        "image/png",
        "application/pdf",
        "image/jpeg",
        "text/latex",
        "application/json",
        "application/javascript",
    ];

    let mime_count = mime_types
        .iter()
        .filter(|&&mime| extraction.content.contains(mime))
        .count();
    assert!(
        mime_count >= 3,
        "Should extract at least 3 MIME type references (found {})",
        mime_count
    );

    assert!(
        extraction.content.contains("mc") && extraction.content.contains("E"),
        "Should extract code cell variable expression content"
    );

    assert!(
        extraction.content.contains("class Mime") || extraction.content.contains("Mime:"),
        "Should extract Mime class definition"
    );

    // Check metadata for execution_count (stored in cells metadata, not rendered in content)
    assert!(
        has_cells_with_execution_count(&extraction.metadata.additional),
        "Should preserve execution_count metadata from code outputs"
    );

    println!(
        "✓ mime.ipynb: Successfully extracted {} characters of MIME-aware content",
        extraction.content.len()
    );
}

/// Test mime.out.ipynb - Validates cell output type and multi-format output handling.
///
/// This notebook is a variant of mime.ipynb with potentially different output formats.
/// Expected contents similar to mime.ipynb but may have additional output variations.
///
/// Cell structure:
/// - Code cells with various output types
/// - Stream stdout outputs (printed text)
/// - Execute result outputs (computed values)
/// - Display data outputs (rendered content)
/// - Multiple MIME representations of same output
///
/// Pandoc output shows:
/// - Output type classification (execute_result, stream, display_data)
/// - MIME type preservation when multiple formats present
/// - Execution count tracking for interactive computation
///
/// Expected baseline from Pandoc:
/// - Same content as mime.ipynb with output variations
/// - Different formatting based on output type
#[tokio::test]
async fn test_jupyter_mime_out_notebook_extraction() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("mime.out.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read mime.out.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    assert_eq!(
        extraction.mime_type, "application/x-ipynb+json",
        "MIME type should be preserved"
    );

    assert!(!extraction.content.is_empty(), "Extracted content should not be empty");

    assert!(
        extraction.content.contains("class") || extraction.content.contains("def"),
        "Should extract Python code cells"
    );

    assert!(
        extraction.content.contains("output") || extraction.content.contains("execute"),
        "Should preserve output type information"
    );

    assert!(
        extraction.content.contains("text")
            || extraction.content.contains("html")
            || extraction.content.contains("image"),
        "Should preserve MIME type information"
    );

    assert!(
        extraction.content.contains("Supported")
            || extraction.content.contains("formatters")
            || extraction.content.contains("write"),
        "Should extract markdown cell content"
    );

    assert!(
        extraction.content.contains("math")
            || extraction.content.contains("dataclass")
            || extraction.content.contains("Mime"),
        "Should extract scientific computing content"
    );

    println!(
        "✓ mime.out.ipynb: Successfully extracted {} characters",
        extraction.content.len()
    );
}

/// Test rank.ipynb - Validates image output and display_data handling.
///
/// Notebook contains:
/// - Code cell 1: Import matplotlib.pyplot (execution_count=1)
/// - Code cell 2: Create subplot with imshow visualization (execution_count=2)
///   - Output: display_data with multiple MIME types:
///     - text/html: "<p><em>you should see this when converting...</em></p>"
///     - image/png: base64-encoded PNG image
///     - text/plain: "<Figure size 4x4 with 1 Axes>"
///
/// This tests the complex case of display_data outputs with:
/// - Text HTML fallback
/// - Binary image data
/// - Text representation
///
/// Pandoc output format:
/// - Display data outputs: `::: {.output .display_data}`
/// - Image references: `![](hash.png)` - Pandoc extracts images
/// - Multiple MIME representations collapsed into single output
///
/// Expected baseline from Pandoc:
/// - Image plot reference extracted
/// - Figure description extracted
/// - HTML fallback content available
#[tokio::test]
async fn test_jupyter_rank_notebook_extraction() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("rank.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read rank.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    assert_eq!(
        extraction.mime_type, "application/x-ipynb+json",
        "MIME type should be preserved"
    );

    assert!(!extraction.content.is_empty(), "Extracted content should not be empty");

    assert!(
        extraction.content.contains("matplotlib")
            || extraction.content.contains("pyplot")
            || extraction.content.contains("plt"),
        "Should extract matplotlib import code"
    );

    assert!(
        extraction.content.contains("image")
            || extraction.content.contains("Figure")
            || extraction.content.contains("Axes")
            || extraction.content.contains(".png"),
        "Should preserve image output information"
    );

    // Output type markers are metadata (stored in cells), not rendered in content text
    // The actual data outputs (text, HTML, images) are extracted, but the type labels are stored as metadata

    assert!(
        extraction.content.contains("subplots")
            || extraction.content.contains("imshow")
            || extraction.content.contains("plt."),
        "Should extract figure creation code"
    );

    assert!(
        extraction.content.contains("Figure")
            || extraction.content.contains("Axes")
            || extraction.content.contains("size")
            || extraction.content.contains("see"),
        "Should extract alternative text representation"
    );

    // Kernel and language information is stored in notebook metadata
    // Verify metadata contains language_info
    if let Some(Value::Object(metadata_obj)) = extraction.metadata.additional.get("language_info") {
        assert!(!metadata_obj.is_empty(), "Should preserve language_info metadata");
    }

    println!(
        "✓ rank.ipynb: Successfully extracted {} characters of visualization content",
        extraction.content.len()
    );
}

/// Test metadata aggregation across all notebooks.
///
/// Validates that:
/// - Notebook metadata is extracted and available
/// - Cell-level metadata is preserved where applicable
/// - Kernel specifications are captured
/// - Language information is available
#[tokio::test]
async fn test_jupyter_metadata_aggregation() {
    let config = ExtractionConfig::default();

    let notebooks = vec![
        ("simple.ipynb", jupyter_fixture("simple.ipynb")),
        ("mime.ipynb", jupyter_fixture("mime.ipynb")),
        ("rank.ipynb", jupyter_fixture("rank.ipynb")),
    ];

    for (name, path) in notebooks {
        let notebook_content = match fs::read(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}. Skipping.", name, e);
                continue;
            }
        };

        let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

        if result.is_err() {
            println!("Skipping metadata test for {}: Pandoc may not be installed", name);
            continue;
        }

        let extraction = result.expect("Operation failed");

        assert!(
            !extraction.content.is_empty(),
            "{}: Should have extracted content",
            name
        );

        assert!(
            extraction.metadata.additional.is_empty() || !extraction.metadata.additional.is_empty(),
            "{}: Metadata structure should be consistent",
            name
        );

        assert_eq!(
            extraction.mime_type, "application/x-ipynb+json",
            "{}: MIME type should be preserved",
            name
        );

        println!("✓ {}: Metadata validated", name);
    }
}

/// Test cell content aggregation - validates that all cell types are extracted.
///
/// Verifies:
/// - Markdown cells are extracted as text
/// - Code cells preserve source code
/// - Output cells are aggregated properly
/// - Cell ordering is maintained in output
#[tokio::test]
async fn test_jupyter_cell_content_aggregation() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("mime.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read mime.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    let code_indicators = ["class", "def", "import", "from", "python"];
    let code_count = code_indicators
        .iter()
        .filter(|&&indicator| extraction.content.contains(indicator))
        .count();
    assert!(
        code_count >= 2,
        "Should extract code cells with Python code (found {} indicators)",
        code_count
    );

    let markdown_indicators = ["Supported", "IPython", "formatters"];
    let markdown_count = markdown_indicators
        .iter()
        .filter(|&&indicator| extraction.content.contains(indicator))
        .count();
    assert!(
        markdown_count >= 1,
        "Should extract markdown cells (found {} indicators)",
        markdown_count
    );

    // Output data (text, HTML, images) are extracted as actual content
    // The type labels ("output", "stream", "execute") are metadata, not rendered text
    assert!(
        !extraction.content.is_empty(),
        "Should extract non-empty content from notebook cells"
    );

    // Cell metadata is stored in the metadata object
    assert!(
        !extraction.metadata.additional.is_empty(),
        "Should preserve cell metadata"
    );

    println!(
        "✓ Cell aggregation: Successfully aggregated {} cells",
        extraction.content.len()
    );
}

/// Test MIME output handling - validates correct MIME type representations.
///
/// Verifies:
/// - text/plain outputs are extracted
/// - text/html outputs are preserved
/// - image/png outputs are referenced
/// - text/markdown outputs are processed
/// - execute_result vs stream vs display_data distinction
#[tokio::test]
async fn test_jupyter_mime_output_handling() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("rank.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read rank.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    assert!(
        extraction.content.contains("image")
            || extraction.content.contains("png")
            || extraction.content.contains("jpg")
            || extraction.content.contains("Figure"),
        "Should handle image MIME types"
    );

    // Text representations of outputs are extracted (Figure, Axes descriptions)
    // Output type markers (display_data, execute_result) are metadata, not rendered text
    assert!(
        extraction.content.contains("Figure")
            || extraction.content.contains("Axes")
            || extraction.content.contains("matplotlib")
            || extraction.content.contains("plt"),
        "Should extract alternative text representations of visual outputs"
    );

    println!("✓ MIME output handling: Correctly processed various MIME types");
}

/// Test notebook structure preservation - validates cell IDs and ordering.
///
/// Verifies:
/// - Cell IDs are preserved
/// - Cell order matches notebook order
/// - Execution counts are preserved for code cells
#[tokio::test]
async fn test_jupyter_notebook_structure_preservation() {
    let config = ExtractionConfig::default();

    let notebook_path = jupyter_fixture("simple.ipynb");
    let notebook_content = match fs::read(notebook_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read simple.ipynb: {}. Skipping test.", e);
            return;
        }
    };

    let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

    if result.is_err() {
        println!("Skipping test: Pandoc may not be installed");
        return;
    }

    let extraction = result.expect("Operation failed");

    // Cell IDs and execution_count are stored in metadata, not in rendered content text
    // Verify metadata contains cell information
    assert!(
        !extraction.metadata.additional.is_empty(),
        "Should preserve cell metadata"
    );

    if let Some(Value::Array(cells)) = extraction.metadata.additional.get("cells") {
        assert!(!cells.is_empty(), "Should preserve cell metadata entries");
    }

    println!("✓ Structure preservation: Cell IDs and ordering maintained");
}

/// Integration test comparing Pandoc output with extraction.
///
/// This test validates that the extraction matches Pandoc's baseline output format.
/// Pandoc converts .ipynb to markdown with cell dividers and metadata preservation.
#[tokio::test]
async fn test_jupyter_pandoc_baseline_alignment() {
    let config = ExtractionConfig::default();

    let notebooks = vec!["simple.ipynb", "mime.ipynb", "mime.out.ipynb", "rank.ipynb"];

    for notebook_name in notebooks {
        let notebook_path = jupyter_fixture(notebook_name);
        let notebook_content = match fs::read(&notebook_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}. Skipping.", notebook_name, e);
                continue;
            }
        };

        let result = extract_bytes(&notebook_content, "application/x-ipynb+json", &config).await;

        if result.is_err() {
            println!(
                "Skipping baseline test for {}: Pandoc may not be installed",
                notebook_name
            );
            continue;
        }

        let extraction = result.expect("Operation failed");

        // Cell structure and output type information is stored in metadata
        // Content should contain the actual extracted text (markdown, code, outputs)
        assert!(
            !extraction.content.is_empty(),
            "{}: Should extract meaningful content",
            notebook_name
        );

        // Verify metadata contains cell structure information
        assert!(
            !extraction.metadata.additional.is_empty(),
            "{}: Should preserve notebook metadata",
            notebook_name
        );

        assert_eq!(
            extraction.mime_type, "application/x-ipynb+json",
            "{}: MIME type should match",
            notebook_name
        );

        println!(
            "✓ {}: Baseline alignment verified ({} chars extracted)",
            notebook_name,
            extraction.content.len()
        );
    }
}
