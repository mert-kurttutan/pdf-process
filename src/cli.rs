use std::{path::PathBuf, process::ExitCode, time::Duration};

use clap::Parser;

use crate::workflow::{WorkflowOptions, convert_pdf};

#[derive(Debug, Parser)]
#[command(
    name = "pdf-process",
    about = "Convert a whole PDF to Datalab HTML and optionally derive Typst."
)]
pub struct ConvertArgs {
    /// Path to the input PDF file.
    pub input_pdf: PathBuf,
    /// Datalab API key. Defaults to `DATALAB_API_KEY`.
    #[arg(long)]
    pub api_key: Option<String>,
    /// Directory for generated files. Defaults to the input PDF directory.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    /// Datalab conversion mode for the whole PDF.
    #[arg(long, value_parser = ["fast", "balanced", "accurate"], default_value = "accurate")]
    pub mode: String,
    /// Also generate a Typst file from the returned HTML.
    #[arg(long)]
    pub typst: bool,
    /// Also generate a `.mathjax.html` browser preview with `MathJax` rendering.
    #[arg(long)]
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

#[must_use]
pub fn run() -> ExitCode {
    let args = ConvertArgs::parse();
    let options = WorkflowOptions {
        api_key: args.api_key,
        mode: args.mode,
        typst: args.typst,
        mathjax_preview: args.mathjax_preview,
        output_dir: args.output_dir,
        save_metadata: !args.no_metadata,
        poll_interval: Duration::from_secs_f64(args.poll_interval),
        timeout: Duration::from_secs_f64(args.timeout),
    };
    match convert_pdf(&args.input_pdf, &options) {
        Ok(output) => {
            println!("HTML: {}", output.html_path.display());
            if let Some(path) = output.mathjax_html_path {
                println!("MathJax HTML: {}", path.display());
            }
            if let Some(path) = output.typst_path {
                println!("Typst: {}", path.display());
            }
            if let Some(path) = output.metadata_path {
                println!("Metadata: {}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
