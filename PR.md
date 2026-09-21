# PR: Add cached conversion artifacts

Introduce a small content-addressed filesystem cache keyed by the SHA-256 of
the input PDF and conversion options. Cache entries live under
`<os-cache>/pdf-process/<key>/` with a completion manifest, so successful
conversions are reused without calling Datalab. `--force` bypasses a matching
entry.

Store each conversion's HTML, MathJax HTML, Typst, metadata, and extracted PNG
files as named artifacts in the cache entry. `--cache-dir` selects the cache
root; otherwise the operating system's per-user cache directory is used.

Acceptance criteria:

- [x] Identical input and options return cached artifacts without an API request.
- [x] Changed inputs/options produce a separate cache entry.
- [x] HTML, PNG, MathJax, Typst, and metadata paths are reported consistently.
- [x] Users can choose a cache directory and force a fresh conversion.
- [x] Cache and artifact writes are staged and atomically renamed; incomplete
  `.partial-*` entries are never treated as cache hits.
