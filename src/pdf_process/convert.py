from __future__ import annotations

from pathlib import Path

from pdf_process.workflow import WorkflowOutput, convert_pdf_workflow


def convert_pdf(
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
    return convert_pdf_workflow(
        input_pdf,
        api_key=api_key,
        mode=mode,
        typst=typst,
        mathjax_preview=mathjax_preview,
        output_dir=output_dir,
        save_metadata=save_metadata,
        poll_interval=poll_interval,
        timeout_seconds=timeout_seconds,
    )
