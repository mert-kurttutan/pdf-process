# pdf-process

Convert PDFs into HTML through the Datalab Convert API, then optionally generate
equivalent Typst from the returned HTML.

The primary workflow is:

1. Send the whole PDF to Datalab `/api/v1/convert` with `output_format=html`.
2. Poll the Datalab result endpoint until conversion completes.
3. Save the returned HTML next to the source PDF, typically under `assets/`.
4. If requested, convert that saved HTML into a `.typ` file next to the PDF.

## Requirements

- Python `>=3.10,<4.0`
- `uv`
- Datalab API key in `DATALAB_API_KEY`
- Optional: `typst` CLI if you want to compile generated `.typ` files

## Install

```sh
uv sync --group dev
```

## Usage

Convert a whole PDF to HTML and save it beside the PDF:

```sh
export DATALAB_API_KEY=...
uv run pdf-process assets/paper.pdf
```

This writes:

```text
assets/paper.html
assets/paper.datalab.json
```

Also generate Typst from the returned HTML:

```sh
uv run pdf-process assets/paper.pdf --typst
```

This also writes:

```text
assets/paper.typ
```

Also create a MathJax browser preview:

```sh
uv run pdf-process assets/paper.pdf --mathjax-preview
```

Create a MathJax preview from an existing HTML file without re-calling Datalab:

```sh
uv run pdf-process-mathjax assets/paper.html
```

This writes:

```text
assets/paper.mathjax.html
```

Choose a Datalab mode:

```sh
uv run pdf-process assets/paper.pdf --mode balanced
```

Use a different output directory:

```sh
uv run pdf-process assets/paper.pdf --output-dir out/paper
```

## Math And Typst

Datalab performs the PDF-to-HTML conversion, including math-oriented parsing
when its model detects formulas. The Typst step is deliberately downstream from
HTML: it reads the returned HTML and maps headings, paragraphs, lists, simple
tables, emphasis, and elements with `class="math"` into Typst syntax.

For higher-fidelity Typst output, improve the HTML-to-Typst mapper rather than
calling Datalab differently. The source of truth remains the returned HTML.

## Development

```sh
uv run ruff check
uv run pytest
```
