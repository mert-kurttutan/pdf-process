from pdf_process.html_to_typst import html_to_typst


def test_html_to_typst_maps_headings_paragraphs_and_math() -> None:
    typst = html_to_typst("<h1>Title</h1><p>Area</p><p><span class='math'>A = pi r^2</span></p>")

    assert "= Title" in typst
    assert "Area" in typst
    assert "$ A = pi r^2 $" in typst


def test_html_to_typst_escapes_typst_control_characters() -> None:
    typst = html_to_typst("<p>#total * [x]</p>")

    assert "\\#total \\* \\[x\\]" in typst

