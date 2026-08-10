from __future__ import annotations

import argparse
from pathlib import Path

from pdf_process.convert import convert_pdf
from pdf_process.datalab import DatalabError
from pdf_process.mathjax_preview import write_mathjax_preview


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="pdf-process",
        description="Convert a whole PDF to Datalab HTML and optionally derive Typst.",
    )
    parser.add_argument("input_pdf", type=Path, help="Path to input PDF file.")
    parser.add_argument(
        "--api-key",
        default=None,
        help="Datalab API key. Defaults to DATALAB_API_KEY.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Directory for generated files. Defaults to the input PDF directory.",
    )
    parser.add_argument(
        "--mode",
        choices=["fast", "balanced", "accurate"],
        default="accurate",
        help="Datalab conversion mode for the whole PDF (default: accurate).",
    )
    parser.add_argument(
        "--typst",
        action="store_true",
        help="Also generate a Typst file from the returned HTML.",
    )
    parser.add_argument(
        "--mathjax-preview",
        action="store_true",
        help="Also generate a .mathjax.html browser preview with MathJax rendering.",
    )
    parser.add_argument(
        "--no-metadata",
        action="store_true",
        help="Do not save the Datalab metadata JSON next to the output.",
    )
    parser.add_argument(
        "--poll-interval",
        type=float,
        default=2.0,
        help="Seconds between result polling attempts (default: 2.0).",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=600.0,
        help="Maximum seconds to wait for Datalab conversion (default: 600).",
    )
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    try:
        output = convert_pdf(
            input_pdf=args.input_pdf,
            api_key=args.api_key,
            mode=args.mode,
            typst=args.typst,
            mathjax_preview=args.mathjax_preview,
            output_dir=args.output_dir,
            save_metadata=not args.no_metadata,
            poll_interval=args.poll_interval,
            timeout_seconds=args.timeout,
        )
    except (DatalabError, FileNotFoundError, ValueError) as error:
        parser.error(str(error))
    print(f"HTML: {output.html_path}")
    if output.mathjax_html_path:
        print(f"MathJax HTML: {output.mathjax_html_path}")
    if output.typst_path:
        print(f"Typst: {output.typst_path}")
    if output.metadata_path:
        print(f"Metadata: {output.metadata_path}")

def preview_main() -> None:
    parser = argparse.ArgumentParser(
        prog="pdf-process-mathjax",
        description="Create a MathJax preview HTML file from an existing Datalab HTML file.",
    )
    parser.add_argument("input_html", type=Path, help="Path to an existing Datalab HTML file.")
    parser.add_argument("--output", type=Path, default=None, help="Output HTML path. Defaults to <stem>.mathjax.html.")
    args = parser.parse_args()

    try:
        output = write_mathjax_preview(args.input_html, args.output)
    except (FileNotFoundError, ValueError) as error:
        parser.error(str(error))
    print(f"MathJax HTML: {output}")


if __name__ == "__main__":
    main()
