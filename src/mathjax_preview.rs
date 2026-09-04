use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

const MATHJAX_URL: &str = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js";
const MATHJAX_INJECTION: &str = r#"  <script>
   window.MathJax = {
    tex: {
     inlineMath: [["\\(", "\\)"]],
     displayMath: [["\\[", "\\]"]]
    },
    startup: {
     ready: () => {
      document.querySelectorAll("math").forEach((node) => {
       const display = node.getAttribute("display") === "block";
       const text = node.textContent.trim();
       node.replaceWith(document.createTextNode(display ? `\\[${text}\\]` : `\\(${text}\\)`));
      });
      MathJax.startup.defaultReady();
     }
    }
   };
  </script>
  <script defer src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js"></script>
"#;

#[derive(Debug, Error)]
pub enum MathJaxError {
    #[error("input file not found: {0}")]
    MissingInput(PathBuf),
    #[error("input file must be HTML")]
    InvalidInput,
    #[error("could not read or write a local file: {0}")]
    Io(#[from] std::io::Error),
}

#[must_use]
pub fn render_mathjax_preview(html: &str) -> String {
    if html.contains(MATHJAX_URL) {
        return html.to_owned();
    }
    if let Some(head_end) = html.to_ascii_lowercase().find("</head>") {
        return format!(
            "{}{}{}",
            &html[..head_end],
            MATHJAX_INJECTION,
            &html[head_end..]
        );
    }
    format!(
        "<!doctype html>\n<html>\n<head>\n  <meta charset=\"utf-8\">\n{MATHJAX_INJECTION}</head>\n<body>\n{html}\n</body>\n</html>\n"
    )
}

/// Write a MathJax-enabled sidecar for an existing HTML file.
///
/// # Errors
///
/// Returns [`MathJaxError`] when the input is absent or not HTML, or when the
/// preview cannot be read or written.
pub fn write_mathjax_preview(
    input_html: &Path,
    output_html: Option<&Path>,
) -> Result<PathBuf, MathJaxError> {
    if !input_html.is_file() {
        return Err(MathJaxError::MissingInput(input_html.to_path_buf()));
    }
    if !input_html.extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
    }) {
        return Err(MathJaxError::InvalidInput);
    }
    let output_path = output_html.map_or_else(
        || {
            let stem = input_html.file_stem().unwrap_or_default().to_string_lossy();
            input_html.with_file_name(format!("{stem}.mathjax.html"))
        },
        Path::to_path_buf,
    );
    fs::write(
        &output_path,
        render_mathjax_preview(&fs::read_to_string(input_html)?),
    )?;
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::render_mathjax_preview;

    #[test]
    fn injects_mathjax_into_an_existing_head() {
        let html = render_mathjax_preview("<html><head></head><body><math>x</math></body></html>");
        assert!(html.contains("mathjax@3"));
        assert!(html.contains("querySelectorAll(\"math\")"));
        assert!(html.contains("<head>"));
    }
}
