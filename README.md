# pdf-process

A Rust CLI that converts PDFs into HTML through the Datalab Convert API, then
optionally generates equivalent Typst from the returned HTML.

The primary workflow is:

1. Send the whole PDF to Datalab `/api/v1/convert` with `output_format=html`.
2. Poll the Datalab result endpoint until conversion completes.
3. Save the returned HTML next to the source PDF, typically under `assets/`.
4. If requested, convert that saved HTML into a `.typ` file next to the PDF.

## Requirements

- Rust `>=1.85` and Cargo
- Datalab API key in `DATALAB_API_KEY`, either as a process environment
  variable or in a `.env` file in the working directory
- Optional: `typst` CLI if you want to compile generated `.typ` files

## Install

```sh
nix develop -c cargo build
```

## Usage

Configuration is loaded into a shared key-value map by `utils`. Values from
`.env` provide defaults, while process environment variables take precedence.
An explicit `--api-key` takes precedence over both.

Convert a whole PDF to HTML and save it beside the PDF:

```sh
export DATALAB_API_KEY=...
cargo run -- html assets/paper.pdf
```

This writes:

```text
assets/paper.html
assets/paper.mathjax.html
assets/paper.datalab.json
```

Also generate Typst from the returned HTML:

```sh
cargo run -- html assets/paper.pdf --typst
```

This also writes:

```text
assets/paper.typ
```

MathJax browser preview generation is enabled by default:

```sh
cargo run -- html assets/paper.pdf
```

Disable the MathJax browser preview when needed:

```sh
cargo run -- html assets/paper.pdf --no-mathjax-preview
```

Choose a Datalab mode:

```sh
cargo run -- html assets/paper.pdf --mode balanced
```

Generated files are placed in a directory named `<stem>.out`, beside the input
PDF by default:

```sh
cargo run -- html assets/paper.pdf
```

This writes all artifacts under `assets/paper.out/`. The command fails if
that directory already exists.

Choose a different parent output directory:

```sh
cargo run -- html assets/paper.pdf --output-dir out
```

This writes all artifacts under `out/paper.out/`.

## Math And Typst

Datalab performs the PDF-to-HTML conversion, including math-oriented parsing
when its model detects formulas. The Typst step is deliberately downstream from
HTML: it reads the returned HTML and maps headings, paragraphs, lists, simple
tables, emphasis, and elements with `class="math"` into Typst syntax.

For higher-fidelity Typst output, improve the HTML-to-Typst mapper rather than
calling Datalab differently. The source of truth remains the returned HTML.

## Development

```sh
nix develop -c cargo fmt --check
nix develop -c cargo clippy --all-targets --all-features -- -D warnings
nix develop -c cargo test
```

The Nix development shell supplies Rust, Cargo, Clippy, Rustfmt, a C toolchain,
and the optional Typst compiler. Enter it interactively with `nix develop`, or
run a one-off command with `nix develop -c <command>`.
