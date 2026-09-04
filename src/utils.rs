//! Application configuration loading.
//!
//! Values from `.env` are used as defaults. Process environment variables are
//! applied afterwards, so they always take precedence over the file.

use std::{collections::HashMap, env, fs, path::Path};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid configuration entry on line {line}")]
    InvalidLine { line: usize },
}

/// Load `.env` and process environment variables into one configuration map.
///
/// A missing `.env` file is allowed. Values from the process environment
/// override values loaded from `.env`.
pub fn load_config() -> Result<HashMap<String, String>, ConfigError> {
    load_config_from(Path::new(".env"), env::vars())
}

fn load_config_from<I>(path: &Path, environment: I) -> Result<HashMap<String, String>, ConfigError>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut config = match fs::read_to_string(path) {
        Ok(contents) => parse_env_file(&contents)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(error) => return Err(error.into()),
    };
    config.extend(environment);
    Ok(config)
}

fn parse_env_file(contents: &str) -> Result<HashMap<String, String>, ConfigError> {
    contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            Some((index + 1, line))
        })
        .map(|(line_number, line)| {
            let line = line.strip_prefix("export ").unwrap_or(line);
            let (key, value) = line
                .split_once('=')
                .ok_or(ConfigError::InvalidLine { line: line_number })?;
            let key = key.trim();
            if key.is_empty() {
                return Err(ConfigError::InvalidLine { line: line_number });
            }
            Ok((key.to_owned(), parse_value(value.trim())))
        })
        .collect()
}

fn parse_value(value: &str) -> String {
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_owned();
        }
    }
    value.split_once(" #").map_or_else(
        || value.to_owned(),
        |(value, _)| value.trim_end().to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::{load_config_from, parse_env_file};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn parses_common_dotenv_entries() {
        let config = parse_env_file("# comment\nexport API_KEY=abc\nNAME=\"demo app\"\n").unwrap();
        assert_eq!(config.get("API_KEY"), Some(&"abc".to_owned()));
        assert_eq!(config.get("NAME"), Some(&"demo app".to_owned()));
    }

    #[test]
    fn process_environment_overrides_dotenv() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pdf-process-config-{unique}.env"));
        fs::write(&path, "DATALAB_API_KEY=from-file\n").unwrap();
        let config = load_config_from(
            &path,
            [("DATALAB_API_KEY".to_owned(), "from-env".to_owned())],
        )
        .unwrap();
        assert_eq!(config.get("DATALAB_API_KEY"), Some(&"from-env".to_owned()));
        fs::remove_file(path).unwrap();
    }
}
