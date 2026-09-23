# Agent Notes

This project is a Rust CLI that converts PDFs into HTML through the Datalab
Convert API.

- Send the whole PDF to Datalab `/api/v1/convert` with `output_format=html`.
- Run the conversion through the single `pdf-process html` subcommand.
- Load configuration from `.env`, then overlay process environment variables;
  process environment variables take precedence. Explicit `--api-key` takes
  precedence over both.
- Do not set `page_range` or `max_pages` for the default workflow.
- Store all generated artifacts in `<stem>.out/` beside the input PDF, or
  under `<output-dir>/<stem>.out/` when `--output-dir` is supplied.
- Save returned HTML, MathJax HTML, metadata, and extracted images in
  that output directory. MathJax HTML is enabled by default.
- Refuse to run when the target output path is a file or a non-empty
  directory; an existing empty output directory may be reused.
- Keep equations semantic. Do not represent math as positioned glyph fragments.

Development commands:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Example:

```sh
cargo run -- html assets/scientific-article.pdf
```

The `assets/` directory keeps source files at its top level; generated asset
subdirectories are ignored by Git.

For Mert Kurttutan's local machine only, after pushing a commit that changes
Rust code, rebuild and install the published local binary, then synchronize
the local helper scripts. Do not run this for documentation-only or other
non-Rust pushes, and do not treat it as a general contributor or CI step:

```sh
nu /home/kmert/projects/nixos-conf/scripts/build-local-cargo-bin.nu
nu /home/kmert/projects/nixos-conf/scripts/sync-local-bin.nu
```

The build helper is named `build-local-cargo-bin.nu` (not
`build-cargo-local-bin`) and builds the latest pushed GitHub repository.
