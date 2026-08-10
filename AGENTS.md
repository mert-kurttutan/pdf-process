# Agent Notes

This project converts PDFs into HTML through the Datalab Convert API, with
optional Typst output generated from the returned HTML.

Use the nearby `/home/kmert/projects/pdf-to-typst` project as workflow context:

- Send the whole PDF to Datalab `/api/v1/convert` with `output_format=html`.
- Do not set `page_range` or `max_pages` for the default workflow.
- Save returned HTML next to the source PDF, usually under `assets/`.
- Save `<stem>.datalab.json` metadata next to the HTML unless disabled.
- Generate `<stem>.typ` from the saved HTML only when requested.
- Keep equations semantic. Do not represent math as positioned glyph fragments.
- If adding Typst authoring behavior, follow `/home/kmert/projects/pdf-to-typst/.skills/typst/SKILL.md`.

Development commands:

```sh
uv run ruff check
uv run pytest
```
