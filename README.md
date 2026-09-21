# pdf-process

A Rust CLI that converts PDFs into HTML through the Datalab Convert API, then
optionally generates equivalent Typst from the returned HTML.

The primary workflow is:

1. Send the whole PDF to Datalab `/api/v1/convert` with `output_format=html`.
2. Poll the Datalab result endpoint until conversion completes.
3. Save the returned HTML and extracted images in the local cache.
4. If requested, convert that saved HTML into a `.typ` artifact.

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

This writes under the cache's artifact directory:

```text
<cache>/<cache-key>/artifacts/paper.html
<cache>/<cache-key>/artifacts/paper.mathjax.html
<cache>/<cache-key>/artifacts/paper.datalab.json
```

Also generate Typst from the returned HTML:

```sh
cargo run -- html assets/paper.pdf --typst
```

This also writes:

```text
<cache>/<cache-key>/artifacts/paper.typ
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

Artifacts are stored in a local content-addressed cache and reused when the
PDF and conversion options match:

```sh
cargo run -- html assets/paper.pdf
```

Use `--force` to bypass a matching cache entry, `--cache-dir` to choose the
cache location. By default, the cache is stored in the operating system's
per-user cache directory. The command reports whether the result was cached
and prints paths for HTML, extracted PNGs, MathJax HTML, Typst, and metadata.
See [docs/cache.md](docs/cache.md) for the cache layout and hashing details.

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
