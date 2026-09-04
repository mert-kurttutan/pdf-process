use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use pdf_process::mathjax_preview::write_mathjax_preview;

#[derive(Debug, Parser)]
#[command(
    name = "pdf-process-mathjax",
    about = "Create a MathJax preview from existing Datalab HTML."
)]
struct Args {
    /// Path to an existing Datalab HTML file.
    input_html: PathBuf,
    /// Output HTML path. Defaults to <stem>.mathjax.html.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match write_mathjax_preview(&args.input_html, args.output.as_deref()) {
        Ok(output) => {
            println!("MathJax HTML: {}", output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
