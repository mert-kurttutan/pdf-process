use std::{
    fs,
    path::{Component, Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use reqwest::{
    blocking::{Client, Response, multipart},
    header::HeaderMap,
};
use serde_json::{Map, Value};
use thiserror::Error;
use url::Url;

pub const API_BASE_URL: &str = "https://www.datalab.to";
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ERROR_DETAILS: usize = 2_000;

#[derive(Debug, Error)]
pub enum DatalabError {
    #[error("input file not found: {0}")]
    MissingInput(PathBuf),
    #[error("input file must be a PDF")]
    InvalidInput,
    #[error("missing Datalab API key; set DATALAB_API_KEY or pass --api-key")]
    MissingApiKey,
    #[error("request to Datalab failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("could not read or write a local file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Datalab returned invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Datalab conversion failed: {0}")]
    Api(String),
    #[error("could not load configuration: {0}")]
    Config(#[from] crate::utils::ConfigError),
    #[error("Datalab response did not include a result check URL")]
    MissingCheckUrl,
    #[error("Datalab conversion completed without HTML output")]
    MissingHtml,
    #[error("timed out waiting for Datalab conversion; last status: {0}")]
    TimedOut(String),
    #[error("Datalab returned an invalid result URL: {0}")]
    InvalidResultUrl(String),
    #[error("Datalab returned an unsafe image path: {0}")]
    UnsafeImagePath(String),
    #[error("Datalab returned invalid base64 image data for {0}")]
    InvalidImage(String),
    #[error("poll interval and timeout must be positive values")]
    InvalidTiming,
}

#[derive(Debug, Clone)]
pub struct DatalabOptions {
    pub api_key: Option<String>,
    pub mode: String,
    pub add_block_ids: bool,
    pub poll_interval: Duration,
    pub timeout: Duration,
}

impl Default for DatalabOptions {
    fn default() -> Self {
        Self {
            api_key: None,
            mode: "accurate".to_owned(),
            add_block_ids: true,
            poll_interval: Duration::from_secs(2),
            timeout: Duration::from_secs(600),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub html: String,
    pub raw_response: Map<String, Value>,
}

/// Convert one whole PDF into Datalab HTML.
///
/// # Errors
///
/// Returns [`DatalabError`] when the input is invalid, the API request fails,
/// the conversion is unsuccessful, or its returned images cannot be saved.
pub fn convert_pdf_to_html(
    input_pdf: &Path,
    options: &DatalabOptions,
) -> Result<ConvertResult, DatalabError> {
    validate_input(input_pdf)?;
    if options.poll_interval.is_zero() || options.timeout.is_zero() {
        return Err(DatalabError::InvalidTiming);
    }

    let config = crate::utils::load_config()?;
    let api_key = options
        .api_key
        .clone()
        .or_else(|| config.get("DATALAB_API_KEY").cloned())
        .filter(|key| !key.trim().is_empty())
        .ok_or(DatalabError::MissingApiKey)?;
    let file_name = input_pdf
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(DatalabError::InvalidInput)?;
    let file = multipart::Part::bytes(fs::read(input_pdf)?)
        .file_name(file_name.to_owned())
        .mime_str("application/pdf")?;
    let form = multipart::Form::new()
        .text("mode", options.mode.clone())
        .text("output_format", "html")
        .text("paginate", "false")
        .text("add_block_ids", bool_form(options.add_block_ids))
        .text("disable_image_extraction", "false")
        .part("file", file);

    let client = Client::builder().timeout(DEFAULT_REQUEST_TIMEOUT).build()?;
    let initial = response_json(
        client
            .post(format!("{API_BASE_URL}/api/v1/convert"))
            .header("X-API-Key", &api_key)
            .multipart(form)
            .send()?,
    )?;
    raise_api_error(&initial)?;
    let check_url = initial
        .get("request_check_url")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            initial
                .get("request_id")
                .and_then(Value::as_str)
                .map(|id| format!("/api/v1/convert/{id}"))
        })
        .ok_or(DatalabError::MissingCheckUrl)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-API-Key",
        api_key
            .parse()
            .map_err(|_| DatalabError::Api("invalid API key header value".to_owned()))?,
    );
    let mut result = poll_result(&client, &absolute_url(&check_url)?, &headers, options)?;
    if result.get("html").and_then(Value::as_str).is_none()
        && let Some(result_url) = result.get("result_url").and_then(Value::as_str)
    {
        result = response_json(client.get(result_url).send()?)?;
    }

    let html = result
        .get("html")
        .and_then(Value::as_str)
        .filter(|html| !html.trim().is_empty())
        .ok_or(DatalabError::MissingHtml)?
        .to_owned();
    save_images(
        input_pdf.parent().unwrap_or_else(|| Path::new(".")),
        result.get("images"),
    )?;
    Ok(ConvertResult {
        html,
        raw_response: result,
    })
}

fn validate_input(input_pdf: &Path) -> Result<(), DatalabError> {
    if !input_pdf.is_file() {
        return Err(DatalabError::MissingInput(input_pdf.to_path_buf()));
    }
    if !input_pdf
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return Err(DatalabError::InvalidInput);
    }
    Ok(())
}

fn response_json(response: Response) -> Result<Map<String, Value>, DatalabError> {
    let status = response.status();
    let body = response.text()?;
    if !status.is_success() {
        return Err(DatalabError::Api(format!("HTTP {status}: {body}")));
    }
    let value: Value = serde_json::from_str(&body)?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| DatalabError::Api("expected a JSON object".to_owned()))
}

fn raise_api_error(data: &Map<String, Value>) -> Result<(), DatalabError> {
    let error = data.get("error").and_then(error_value_message);
    if data.get("success").and_then(Value::as_bool) == Some(false) || error.is_some() {
        return Err(DatalabError::Api(api_error_message(
            data,
            error.as_deref(),
            "Datalab reported an unsuccessful response",
        )));
    }
    Ok(())
}

fn poll_result(
    client: &Client,
    check_url: &Url,
    headers: &HeaderMap,
    options: &DatalabOptions,
) -> Result<Map<String, Value>, DatalabError> {
    let deadline = Instant::now() + options.timeout;
    let mut last_status = "unknown".to_owned();
    while Instant::now() < deadline {
        let result = response_json(
            client
                .get(check_url.clone())
                .headers(headers.clone())
                .send()?,
        )?;
        raise_api_error(&result)?;
        let status = result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !status.is_empty() {
            last_status.clone_from(&status);
        }
        if status == "complete" {
            return Ok(result);
        }
        if matches!(
            status.as_str(),
            "failed" | "error" | "cancelled" | "canceled"
        ) {
            return Err(DatalabError::Api(api_error_message(
                &result,
                result.get("error").and_then(error_value_message).as_deref(),
                &format!("Datalab conversion ended with status: {status}"),
            )));
        }
        thread::sleep(options.poll_interval);
    }
    Err(DatalabError::TimedOut(last_status))
}

fn absolute_url(url: &str) -> Result<Url, DatalabError> {
    if let Ok(url) = Url::parse(url) {
        return Ok(url);
    }
    Url::parse(API_BASE_URL)
        .and_then(|base| base.join(url))
        .map_err(|_| DatalabError::InvalidResultUrl(url.to_owned()))
}

fn bool_form(value: bool) -> String {
    (if value { "true" } else { "false" }).to_owned()
}

fn error_value_message(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(message) if message.trim().is_empty() => None,
        Value::String(message) => Some(message.to_owned()),
        Value::Object(object) => object
            .get("message")
            .or_else(|| object.get("detail"))
            .and_then(error_value_message)
            .or_else(|| compact_json(value)),
        _ => compact_json(value),
    }
}

fn api_error_message(
    data: &Map<String, Value>,
    explicit_error: Option<&str>,
    fallback: &str,
) -> String {
    let message = explicit_error
        .map(str::to_owned)
        .or_else(|| data.get("message").and_then(error_value_message))
        .or_else(|| data.get("detail").and_then(error_value_message))
        .or_else(|| data.get("errors").and_then(error_value_message));
    let response = compact_json(&Value::Object(data.clone()));
    match (message, response) {
        (Some(message), Some(response)) => format!("{message}. Response: {response}"),
        (Some(message), None) => message,
        (None, Some(response)) => format!("{fallback}. Response: {response}"),
        (None, None) => fallback.to_owned(),
    }
}

fn compact_json(value: &Value) -> Option<String> {
    let serialized = serde_json::to_string(value).ok()?;
    if serialized.chars().count() <= MAX_ERROR_DETAILS {
        Some(serialized)
    } else {
        let truncated: String = serialized.chars().take(MAX_ERROR_DETAILS).collect();
        Some(format!("{truncated}… (truncated)"))
    }
}

fn save_images(output_dir: &Path, images: Option<&Value>) -> Result<(), DatalabError> {
    let Some(images) = images.and_then(Value::as_object) else {
        return Ok(());
    };
    for (name, encoded) in images {
        let encoded = encoded
            .as_str()
            .ok_or_else(|| DatalabError::InvalidImage(name.clone()))?;
        let relative_path = Path::new(name);
        if relative_path.is_absolute()
            || relative_path.components().any(|part| {
                matches!(
                    part,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(DatalabError::UnsafeImagePath(name.clone()));
        }
        let image_path = output_dir.join(relative_path);
        if let Some(parent) = image_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = BASE64
            .decode(encoded)
            .map_err(|_| DatalabError::InvalidImage(name.clone()))?;
        fs::write(image_path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{absolute_url, bool_form, raise_api_error};
    use serde_json::{Map, json};

    #[test]
    fn resolves_relative_result_urls() {
        assert_eq!(
            absolute_url("/api/v1/convert/abc").unwrap().as_str(),
            "https://www.datalab.to/api/v1/convert/abc"
        );
    }

    #[test]
    fn writes_api_boolean_fields_as_form_values() {
        assert_eq!(bool_form(true), "true");
        assert_eq!(bool_form(false), "false");
    }

    #[test]
    fn accepts_a_successful_response_with_a_null_error_field() {
        let data: Map<_, _> = [
            ("success".to_owned(), json!(true)),
            ("error".to_owned(), json!(null)),
        ]
        .into_iter()
        .collect();
        assert!(raise_api_error(&data).is_ok());
    }

    #[test]
    fn preserves_failure_details_in_the_reported_error() {
        let data: Map<_, _> = [
            ("success".to_owned(), json!(false)),
            ("message".to_owned(), json!("invalid API key")),
            ("error".to_owned(), json!(null)),
        ]
        .into_iter()
        .collect();
        let error = raise_api_error(&data).unwrap_err().to_string();
        assert!(error.contains("invalid API key"));
        assert!(error.contains("\"success\":false"));
    }
}
