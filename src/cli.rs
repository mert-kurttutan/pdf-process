use std::{path::PathBuf, time::Duration};

use clap::{ArgAction, Parser, Subcommand};

use crate::workflow::{WorkflowError, WorkflowOptions, clean_cache, convert_pdf};

#[derive(Debug, Parser)]
#[command(
    name = "pdf-process",
    about = "Process documents through the Datalab API."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Convert a whole PDF to Datalab HTML and optionally derive Typst.
    Html(HtmlArgs),
    /// Manage cached conversions.
    Cache(CacheArgs),
}

#[derive(Debug, clap::Args)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Debug, Subcommand)]
pub enum CacheCommands {
    /// Remove the entire pdf-process cache directory.
    Clean(CleanArgs),
}

#[derive(Debug, clap::Args)]
pub struct CleanArgs {
    /// Directory containing the local conversion cache.
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct HtmlArgs {
    /// Path to the input PDF file.
    pub input_pdf: PathBuf,
    /// Datalab API key. Defaults to `DATALAB_API_KEY`.
    #[arg(long)]
    pub api_key: Option<String>,
    /// Parent directory for `<pdf-stem>.out`.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    /// Directory containing the local conversion cache.
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,
    /// Reconvert the PDF and replace an existing output directory.
    #[arg(long)]
    pub force: bool,
    /// Datalab conversion mode for the whole PDF.
    #[arg(long, value_parser = ["fast", "balanced", "accurate"], default_value = "accurate")]
    pub mode: String,
    /// Also generate a Typst file from the returned HTML.
    #[arg(long)]
    pub typst: bool,
    /// Also save Datalab's block-level JSON output.
    #[arg(long)]
    pub json: bool,
    /// Do not generate the `.mathjax.html` browser preview.
    #[arg(long = "no-mathjax-preview", action = ArgAction::SetFalse, default_value_t = true)]
    pub mathjax_preview: bool,
    /// Do not save Datalab metadata JSON next to the output.
    #[arg(long)]
    pub no_metadata: bool,
    /// Seconds between result polling attempts.
    #[arg(long, default_value_t = 2.0, value_parser = positive_seconds)]
    pub poll_interval: f64,
    /// Maximum seconds to wait for Datalab conversion.
    #[arg(long, default_value_t = 600.0, value_parser = positive_seconds)]
    pub timeout: f64,
}

fn positive_seconds(value: &str) -> Result<f64, String> {
    let seconds = value
        .parse::<f64>()
        .map_err(|_| "must be a number of seconds".to_owned())?;
    if seconds.is_finite() && seconds > 0.0 {
        Ok(seconds)
    } else {
        Err("must be a positive finite number of seconds".to_owned())
    }
}

/// # Errors
///
/// Returns an error when argument parsing or PDF processing fails.
pub fn run() -> Result<(), WorkflowError> {
    match Cli::parse().command {
        Commands::Html(args) => run_html(args),
        Commands::Cache(CacheArgs {
            command: CacheCommands::Clean(args),
        }) => {
            let cache_dir = clean_cache(args.cache_dir.as_deref())?;
            println!("Cleared cache: {}", cache_dir.display());
            Ok(())
        }
    }
}

fn run_html(args: HtmlArgs) -> Result<(), WorkflowError> {
    let options = WorkflowOptions {
        api_key: args.api_key,
        output_dir: args.output_dir,
        cache_dir: args.cache_dir,
        force: args.force,
        mode: args.mode,
        typst: args.typst,
        json: args.json,
        mathjax_preview: args.mathjax_preview,
        save_metadata: !args.no_metadata,
        poll_interval: Duration::from_secs_f64(args.poll_interval),
        timeout: Duration::from_secs_f64(args.timeout),
    };
    let output = convert_pdf(&args.input_pdf, &options)?;
    println!("Artifacts: {}", output.artifact_dir.display());
    println!("Cache: {}", if output.cached { "hit" } else { "miss" });
    println!("HTML: {}", output.html_path.display());
    for path in output.image_paths {
        println!("Image: {}", path.display());
    }
    if let Some(path) = output.mathjax_html_path {
        println!("MathJax HTML: {}", path.display());
    }
    if let Some(path) = output.typst_path {
        println!("Typst: {}", path.display());
    }
    if let Some(path) = output.json_path {
        println!("JSON: {}", path.display());
    }
    if let Some(path) = output.metadata_path {
        println!("Metadata: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CacheCommands, Cli, Commands};
    use clap::Parser;

    #[test]
    fn mathjax_preview_is_enabled_by_default() {
        let Commands::Html(args) = Cli::try_parse_from(["pdf-process", "html", "document.pdf"])
            .unwrap()
            .command
        else {
            panic!("expected html command");
        };
        assert!(args.mathjax_preview);
    }

    #[test]
    fn mathjax_preview_can_be_disabled() {
        let Commands::Html(args) = Cli::try_parse_from([
            "pdf-process",
            "html",
            "document.pdf",
            "--no-mathjax-preview",
        ])
        .unwrap()
        .command
        else {
            panic!("expected html command");
        };
        assert!(!args.mathjax_preview);
    }

    #[test]
    fn json_output_is_opt_in() {
        let Commands::Html(default) = Cli::try_parse_from(["pdf-process", "html", "document.pdf"])
            .unwrap()
            .command
        else {
            panic!("expected html command");
        };
        assert!(!default.json);

        let Commands::Html(with_json) =
            Cli::try_parse_from(["pdf-process", "html", "document.pdf", "--json"])
                .unwrap()
                .command
        else {
            panic!("expected html command");
        };
        assert!(with_json.json);
    }

    #[test]
    fn parses_cache_clean_with_custom_directory() {
        let cli = Cli::try_parse_from([
            "pdf-process",
            "cache",
            "clean",
            "--cache-dir",
            "/tmp/my-cache",
        ])
        .unwrap();
        let Commands::Cache(cache) = cli.command else {
            panic!("expected cache command");
        };
        let CacheCommands::Clean(args) = cache.command;
        assert_eq!(
            args.cache_dir.unwrap(),
            std::path::PathBuf::from("/tmp/my-cache")
        );
    }
}
