from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from pdf_process.datalab import ConvertResult, convert_pdf_to_html
from pdf_process.html_to_typst import html_to_typst
from pdf_process.mathjax_preview import write_mathjax_preview


@dataclass(frozen=True)
class WorkflowOutput:
    html_path: Path
    mathjax_html_path: Path | None
    typst_path: Path | None
    metadata_path: Path | None


def convert_pdf_workflow(
    input_pdf: Path,
    *,
    api_key: str | None = None,
    mode: str = "accurate",
    typst: bool = False,
    mathjax_preview: bool = False,
    output_dir: Path | None = None,
    save_metadata: bool = True,
    poll_interval: float = 2.0,
    timeout_seconds: float = 600.0,
) -> WorkflowOutput:
    result = convert_pdf_to_html(
        input_pdf,
        api_key=api_key,
        mode=mode,
        poll_interval=poll_interval,
        timeout_seconds=timeout_seconds,
    )
    target_dir = output_dir or input_pdf.parent
    target_dir.mkdir(parents=True, exist_ok=True)

    html_path = target_dir / input_pdf.with_suffix(".html").name
    html_path.write_text(result.html, encoding="utf-8")

    mathjax_html_path = None
    if mathjax_preview:
        mathjax_html_path = write_mathjax_preview(html_path)

    typst_path = None
    if typst:
        typst_path = target_dir / input_pdf.with_suffix(".typ").name
        typst_path.write_text(html_to_typst(result.html), encoding="utf-8")

    metadata_path = None
    if save_metadata:
        metadata_path = target_dir / f"{input_pdf.stem}.datalab.json"
        metadata_path.write_text(_metadata_json(result), encoding="utf-8")

    return WorkflowOutput(
        html_path=html_path,
        mathjax_html_path=mathjax_html_path,
        typst_path=typst_path,
        metadata_path=metadata_path,
    )


def _metadata_json(result: ConvertResult) -> str:
    metadata = {
        "request": {
            key: value
            for key, value in result.raw_response.items()
            if key
            in {
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
            }
        }
    }
    return json.dumps(metadata, indent=2, ensure_ascii=False) + "\n"
