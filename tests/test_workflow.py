from pathlib import Path

from pdf_process import workflow
from pdf_process.datalab import ConvertResult


def test_convert_pdf_workflow_saves_html_metadata_and_typst(tmp_path: Path, monkeypatch) -> None:
    pdf_path = tmp_path / "paper.pdf"
    pdf_path.write_bytes(b"%PDF sample")

    def fake_convert_pdf_to_html(*args, **kwargs) -> ConvertResult:
        return ConvertResult(
            html="<h1>Paper</h1><p><span class='math'>A = pi r^2</span></p>",
            raw_response={"status": "complete", "success": True, "page_count": 3},
        )

    monkeypatch.setattr(workflow, "convert_pdf_to_html", fake_convert_pdf_to_html)

    output = workflow.convert_pdf_workflow(pdf_path, typst=True, mathjax_preview=True)

    assert output.html_path == tmp_path / "paper.html"
    assert output.mathjax_html_path == tmp_path / "paper.mathjax.html"
    assert output.typst_path == tmp_path / "paper.typ"
    assert output.metadata_path == tmp_path / "paper.datalab.json"
    assert output.html_path.read_text(encoding="utf-8").startswith("<h1>Paper</h1>")
    assert "mathjax@3" in output.mathjax_html_path.read_text(encoding="utf-8")
    assert "$ A = pi r^2 $" in output.typst_path.read_text(encoding="utf-8")
    assert '"page_count": 3' in output.metadata_path.read_text(encoding="utf-8")


def test_convert_pdf_workflow_can_write_to_output_dir(tmp_path: Path, monkeypatch) -> None:
    pdf_path = tmp_path / "assets" / "paper.pdf"
    pdf_path.parent.mkdir()
    pdf_path.write_bytes(b"%PDF sample")
    output_dir = tmp_path / "out"

    def fake_convert_pdf_to_html(*args, **kwargs) -> ConvertResult:
        return ConvertResult(html="<p>Done</p>", raw_response={"status": "complete", "success": True})

    monkeypatch.setattr(workflow, "convert_pdf_to_html", fake_convert_pdf_to_html)

    output = workflow.convert_pdf_workflow(pdf_path, output_dir=output_dir, save_metadata=False)

    assert output.html_path == output_dir / "paper.html"
    assert output.mathjax_html_path is None
    assert output.typst_path is None
    assert output.metadata_path is None
