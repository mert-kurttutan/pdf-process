# Cache and hashing

`pdf-process` stores completed conversions in a local filesystem cache. The
cache is the canonical artifact location; there is no separate output
directory.

## Cache location

Use `--cache-dir` to override the cache root:

```sh
pdf-process html document.pdf --cache-dir /path/to/pdf-cache
```

Without an override, the default is the per-user OS cache directory:

- Linux: `$XDG_CACHE_HOME/pdf-process`, or `~/.cache/pdf-process`
- macOS: `~/Library/Caches/pdf-process`
- Windows: `%LOCALAPPDATA%/pdf-process`

## Cache key

The key is the lowercase hexadecimal SHA-256 digest of:

1. The exact bytes of the input PDF.
2. A NUL separator followed by the conversion options:

   ```text
   mode=<mode>
   typst=<true|false>
   mathjax=<true|false>
   metadata=<true|false>
   ```

The API key, input path, filename, and modification time are not included.
Changing the PDF or any listed option creates a different cache entry.

## Layout

```text
<cache-root>/<cache-key>/
├── manifest.json
└── artifacts/
    ├── <stem>.html
    ├── <stem>.mathjax.html
    ├── <stem>.typ
    ├── <stem>.datalab.json
    └── <returned-image-paths>.png
```

Optional artifacts are present only when requested. A cache entry is usable
only after its manifest and artifact directory have been written completely.

## Integrity and refresh

Artifacts are written in a temporary staging directory and renamed into the
cache only after conversion and derived-file generation succeed. Individual
files are also written through temporary files and atomic renames. Incomplete
`.partial-*` directories are never treated as cache hits.

Use `--force` to bypass an existing matching entry and perform a new Datalab
conversion.
