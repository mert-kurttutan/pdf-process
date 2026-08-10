from pathlib import Path

from pdf_process.mathjax_preview import render_mathjax_preview, write_mathjax_preview


def test_render_mathjax_preview_injects_into_head() -> None:
    html = render_mathjax_preview("<html><head></head><body><math>x</math></body></html>")

    assert "mathjax@3" in html
    assert "querySelectorAll(\"math\")" in html
    assert "<head>" in html


def test_write_mathjax_preview_defaults_to_sidecar(tmp_path: Path) -> None:
    html_path = tmp_path / "paper.html"
    html_path.write_text("<p><math>y = x</math></p>", encoding="utf-8")

    output = write_mathjax_preview(html_path)

    assert output == tmp_path / "paper.mathjax.html"
    assert "mathjax@3" in output.read_text(encoding="utf-8")
