use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::{
    datalab::{ConvertResult, DatalabError, DatalabOptions, convert_pdf_to_html},
    html_to_typst::html_to_typst,
    mathjax_preview::{MathJaxError, write_mathjax_preview},
};

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error(transparent)]
    Datalab(#[from] DatalabError),
    #[error(transparent)]
    MathJax(#[from] MathJaxError),
    #[error("could not read or write a local file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not serialize Datalab metadata: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct WorkflowOptions {
    pub api_key: Option<String>,
    pub mode: String,
    pub typst: bool,
    pub mathjax_preview: bool,
    pub output_dir: Option<PathBuf>,
    pub save_metadata: bool,
    pub poll_interval: Duration,
    pub timeout: Duration,
}

impl Default for WorkflowOptions {
    fn default() -> Self {
        Self {
            api_key: None,
            mode: "accurate".to_owned(),
            typst: false,
            mathjax_preview: true,
            output_dir: None,
            save_metadata: true,
            poll_interval: Duration::from_secs(2),
            timeout: Duration::from_secs(600),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowOutput {
    pub html_path: PathBuf,
    pub mathjax_html_path: Option<PathBuf>,
    pub typst_path: Option<PathBuf>,
    pub metadata_path: Option<PathBuf>,
}

/// Convert a PDF and save its requested output artifacts.
///
/// # Errors
///
/// Returns [`WorkflowError`] if conversion, output writing, or metadata
/// serialization fails.
pub fn convert_pdf(
    input_pdf: &Path,
    options: &WorkflowOptions,
) -> Result<WorkflowOutput, WorkflowError> {
    let datalab_options = DatalabOptions {
        api_key: options.api_key.clone(),
        mode: options.mode.clone(),
        poll_interval: options.poll_interval,
        timeout: options.timeout,
        ..DatalabOptions::default()
    };
    let result = convert_pdf_to_html(input_pdf, &datalab_options)?;
    write_outputs(input_pdf, options, &result)
}

fn write_outputs(
    input_pdf: &Path,
    options: &WorkflowOptions,
    result: &ConvertResult,
) -> Result<WorkflowOutput, WorkflowError> {
    let target_dir = options
        .output_dir
        .as_deref()
        .unwrap_or_else(|| input_pdf.parent().unwrap_or_else(|| Path::new(".")));
    fs::create_dir_all(target_dir)?;
    let stem = input_pdf.file_stem().unwrap_or_default().to_string_lossy();
    let html_path = target_dir.join(format!("{stem}.html"));
    fs::write(&html_path, &result.html)?;

    let mathjax_html_path = options
        .mathjax_preview
        .then(|| write_mathjax_preview(&html_path, None))
        .transpose()?;
    let typst_path = if options.typst {
        let path = target_dir.join(format!("{stem}.typ"));
        fs::write(&path, html_to_typst(&result.html))?;
        Some(path)
    } else {
        None
    };
    let metadata_path = if options.save_metadata {
        let path = target_dir.join(format!("{stem}.datalab.json"));
        fs::write(&path, metadata_json(result)?)?;
        Some(path)
    } else {
        None
    };
    Ok(WorkflowOutput {
        html_path,
        mathjax_html_path,
        typst_path,
        metadata_path,
    })
}

fn metadata_json(result: &ConvertResult) -> Result<String, serde_json::Error> {
    let selected: Map<String, Value> = [
        "status",
        "output_format",
        "metadata",
        "success",
        "parse_quality_score",
        "page_count",
        "cost_breakdown",
        "runtime",
        "checkpoint_id",
        "versions",
    ]
    .into_iter()
    .filter_map(|key| {
        result
            .raw_response
            .get(key)
            .cloned()
            .map(|value| (key.to_owned(), value))
    })
    .collect();
    serde_json::to_string_pretty(&json!({ "request": selected })).map(|json| format!("{json}\n"))
}

#[cfg(test)]
mod tests {
    use super::{WorkflowOptions, metadata_json};
    use crate::datalab::ConvertResult;
    use serde_json::json;

    #[test]
    fn workflow_defaults_match_the_cli_contract() {
        let options = WorkflowOptions::default();
        assert_eq!(options.mode, "accurate");
        assert!(options.save_metadata);
        assert!(!options.typst);
        assert!(options.mathjax_preview);
    }

    #[test]
    fn metadata_keeps_documented_response_fields() {
        let result = ConvertResult {
            html: "<p>Done</p>".to_owned(),
            raw_response: [
                ("status".to_owned(), json!("complete")),
                ("page_count".to_owned(), json!(3)),
                ("html".to_owned(), json!("ignored")),
            ]
            .into_iter()
            .collect(),
        };
        let metadata = metadata_json(&result).unwrap();
        assert!(metadata.contains("\"page_count\": 3"));
        assert!(!metadata.contains("ignored"));
    }
}
