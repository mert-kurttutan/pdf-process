use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Duration,
};

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    datalab::{ConvertResult, DatalabError, DatalabOptions, convert_pdf_to_html},
    html_to_typst::html_to_typst,
    mathjax_preview::render_mathjax_preview,
};

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error(transparent)]
    Datalab(#[from] DatalabError),
    #[error(transparent)]
    MathJax(#[from] crate::mathjax_preview::MathJaxError),
    #[error("could not read or write a local file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not serialize Datalab metadata: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct WorkflowOptions {
    pub api_key: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub force: bool,
    pub mode: String,
    pub typst: bool,
    pub mathjax_preview: bool,
    pub save_metadata: bool,
    pub poll_interval: Duration,
    pub timeout: Duration,
}

impl Default for WorkflowOptions {
    fn default() -> Self {
        Self {
            api_key: None,
            cache_dir: None,
            force: false,
            mode: "accurate".to_owned(),
            typst: false,
            mathjax_preview: true,
            save_metadata: true,
            poll_interval: Duration::from_secs(2),
            timeout: Duration::from_secs(600),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowOutput {
    pub artifact_dir: PathBuf,
    pub html_path: PathBuf,
    pub image_paths: Vec<PathBuf>,
    pub mathjax_html_path: Option<PathBuf>,
    pub typst_path: Option<PathBuf>,
    pub metadata_path: Option<PathBuf>,
    pub cached: bool,
}

/// Convert a PDF, reusing a completed local cache entry when possible.
///
/// # Errors
///
/// Returns an error when the input cannot be read, the cache or output cannot
/// be written, or the Datalab conversion fails.
pub fn convert_pdf(
    input_pdf: &Path,
    options: &WorkflowOptions,
) -> Result<WorkflowOutput, WorkflowError> {
    let cache_root = cache_directory(input_pdf, options);
    fs::create_dir_all(&cache_root)?;
    let cache_key = cache_key(input_pdf, options)?;
    let cache_entry = cache_root.join(&cache_key);

    if !options.force && cache_entry_is_complete(&cache_entry) {
        return output_paths(&cache_entry.join("artifacts"), input_pdf, options, true);
    }

    let staging_dir = staging_directory(&cache_root, &cache_key)?;
    let artifacts_dir = staging_dir.join("artifacts");
    fs::create_dir(&artifacts_dir)?;
    let datalab_options = DatalabOptions {
        api_key: options.api_key.clone(),
        mode: options.mode.clone(),
        poll_interval: options.poll_interval,
        timeout: options.timeout,
        ..DatalabOptions::default()
    };
    let result = convert_pdf_to_html(input_pdf, &artifacts_dir, &datalab_options)?;
    write_outputs(&artifacts_dir, input_pdf, options, &result)?;
    write_manifest(&staging_dir, &cache_key)?;

    if cache_entry.exists() {
        fs::remove_dir_all(&cache_entry)?;
    }
    fs::rename(&staging_dir, &cache_entry)?;
    output_paths(&cache_entry.join("artifacts"), input_pdf, options, false)
}

fn cache_directory(_input_pdf: &Path, options: &WorkflowOptions) -> PathBuf {
    options
        .cache_dir
        .clone()
        .unwrap_or_else(default_cache_directory)
}

#[cfg(target_os = "linux")]
fn default_cache_directory() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| home_directory().map(|home| home.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("pdf-process")
}

#[cfg(target_os = "macos")]
fn default_cache_directory() -> PathBuf {
    home_directory()
        .map(|home| home.join("Library").join("Caches"))
        .unwrap_or_else(std::env::temp_dir)
        .join("pdf-process")
}

#[cfg(target_os = "windows")]
fn default_cache_directory() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(home_directory)
        .unwrap_or_else(std::env::temp_dir)
        .join("pdf-process")
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn default_cache_directory() -> PathBuf {
    std::env::temp_dir().join("pdf-process")
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn cache_key(input_pdf: &Path, options: &WorkflowOptions) -> Result<String, WorkflowError> {
    let mut hasher = Sha256::new();
    hasher.update(fs::read(input_pdf)?);
    hasher.update(format!(
        "\0mode={}\0typst={}\0mathjax={}\0metadata={}",
        options.mode, options.typst, options.mathjax_preview, options.save_metadata
    ));
    Ok(hex_digest(&hasher.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut digest = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(digest, "{byte:02x}").expect("writing to a String cannot fail");
    }
    digest
}

fn staging_directory(cache_root: &Path, key: &str) -> Result<PathBuf, WorkflowError> {
    for attempt in 0..100 {
        let candidate = cache_root.join(format!(".{key}.partial-{attempt}"));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "too many cache staging directories",
    )
    .into())
}

fn cache_entry_is_complete(entry: &Path) -> bool {
    entry.join("manifest.json").is_file()
        && entry.join("artifacts").is_dir()
        && fs::read_dir(entry.join("artifacts"))
            .ok()
            .is_some_and(|mut files| files.next().is_some())
}

fn write_manifest(directory: &Path, key: &str) -> Result<(), WorkflowError> {
    let manifest = serde_json::to_string_pretty(&json!({
        "cache_key": key,
        "format": 1,
        "complete": true,
    }))? + "\n";
    write_atomic(&directory.join("manifest.json"), manifest.as_bytes())
}

fn write_outputs(
    target_dir: &Path,
    input_pdf: &Path,
    options: &WorkflowOptions,
    result: &ConvertResult,
) -> Result<(), WorkflowError> {
    let stem = input_pdf.file_stem().unwrap_or_default().to_string_lossy();
    write_atomic(
        &target_dir.join(format!("{stem}.html")),
        result.html.as_bytes(),
    )?;
    if options.mathjax_preview {
        write_atomic(
            &target_dir.join(format!("{stem}.mathjax.html")),
            render_mathjax_preview(&result.html).as_bytes(),
        )?;
    }
    if options.typst {
        write_atomic(
            &target_dir.join(format!("{stem}.typ")),
            html_to_typst(&result.html).as_bytes(),
        )?;
    }
    if options.save_metadata {
        write_atomic(
            &target_dir.join(format!("{stem}.datalab.json")),
            metadata_json(result)?.as_bytes(),
        )?;
    }
    Ok(())
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), WorkflowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!(
        "{}partial-{}",
        path.extension().map_or_else(String::new, |extension| {
            format!("{}.", extension.to_string_lossy())
        }),
        std::process::id()
    ));
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn output_paths(
    target_dir: &Path,
    input_pdf: &Path,
    options: &WorkflowOptions,
    cached: bool,
) -> Result<WorkflowOutput, WorkflowError> {
    let stem = input_pdf.file_stem().unwrap_or_default().to_string_lossy();
    let html_path = target_dir.join(format!("{stem}.html"));
    let mathjax_html_path = options
        .mathjax_preview
        .then(|| target_dir.join(format!("{stem}.mathjax.html")));
    let typst_path = options
        .typst
        .then(|| target_dir.join(format!("{stem}.typ")));
    let metadata_path = options
        .save_metadata
        .then(|| target_dir.join(format!("{stem}.datalab.json")));
    let image_paths = collect_images(target_dir)?;
    Ok(WorkflowOutput {
        artifact_dir: target_dir.to_path_buf(),
        html_path,
        image_paths,
        mathjax_html_path,
        typst_path,
        metadata_path,
        cached,
    })
}

fn collect_images(directory: &Path) -> Result<Vec<PathBuf>, WorkflowError> {
    let mut images = Vec::new();
    collect_images_recursive(directory, &mut images)?;
    images.sort();
    Ok(images)
}

fn collect_images_recursive(
    directory: &Path,
    images: &mut Vec<PathBuf>,
) -> Result<(), WorkflowError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_images_recursive(&path, images)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        {
            images.push(path);
        }
    }
    Ok(())
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
    use super::{WorkflowOptions, cache_directory, cache_key, metadata_json};
    use crate::datalab::ConvertResult;
    use serde_json::json;
    use std::{fs, path::Path};

    #[test]
    fn workflow_defaults_match_the_cli_contract() {
        let options = WorkflowOptions::default();
        assert_eq!(options.mode, "accurate");
        assert!(options.save_metadata);
        assert!(options.mathjax_preview);
        assert!(!options.force);
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

    #[test]
    fn cache_key_changes_when_input_or_options_change() {
        let path =
            std::env::temp_dir().join(format!("pdf-process-cache-key-{}", std::process::id()));
        fs::write(&path, b"pdf").unwrap();
        let first = cache_key(&path, &WorkflowOptions::default()).unwrap();
        let options = WorkflowOptions {
            typst: true,
            ..WorkflowOptions::default()
        };
        let second = cache_key(&path, &options).unwrap();
        fs::write(&path, b"different pdf").unwrap();
        let third = cache_key(&path, &WorkflowOptions::default()).unwrap();
        fs::remove_file(path).unwrap();
        assert_ne!(first, second);
        assert_ne!(first, third);
    }

    #[test]
    fn default_cache_location_is_independent_of_input_and_output() {
        let first = WorkflowOptions::default();
        let second = WorkflowOptions::default();
        assert_eq!(
            cache_directory(Path::new("/tmp/documents/paper.pdf"), &first),
            cache_directory(Path::new("/other/place/paper.pdf"), &second)
        );
    }
}
