use std::path::PathBuf;

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

fn main() -> Result<(), pdf_process::mathjax_preview::MathJaxError> {
    let args = Args::parse();
    let output = write_mathjax_preview(&args.input_html, args.output.as_deref())?;
    println!("MathJax HTML: {}", output.display());
    Ok(())
}
