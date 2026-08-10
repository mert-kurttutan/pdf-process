from __future__ import annotations

from pathlib import Path

MATHJAX_INJECTION = """  <script>
   window.MathJax = {
    tex: {
     inlineMath: [["\\\\(", "\\\\)"]],
     displayMath: [["\\\\[", "\\\\]"]]
    },
    startup: {
     ready: () => {
      document.querySelectorAll("math").forEach((node) => {
       const display = node.getAttribute("display") === "block";
       const text = node.textContent.trim();
       node.replaceWith(document.createTextNode(display ? `\\\\[${text}\\\\]` : `\\\\(${text}\\\\)`));
      });
      MathJax.startup.defaultReady();
     }
    }
   };
  </script>
  <script defer src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js"></script>
"""


def render_mathjax_preview(html: str) -> str:
    if "cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js" in html:
        return html

    lower_html = html.lower()
    head_end = lower_html.find("</head>")
    if head_end != -1:
        return html[:head_end] + MATHJAX_INJECTION + html[head_end:]

    return (
        "<!doctype html>\n"
        "<html>\n"
        "<head>\n"
        '  <meta charset="utf-8">\n'
        f"{MATHJAX_INJECTION}"
        "</head>\n"
        "<body>\n"
        f"{html}\n"
        "</body>\n"
        "</html>\n"
    )


def write_mathjax_preview(input_html: Path, output_html: Path | None = None) -> Path:
    if input_html.suffix.lower() not in {".html", ".htm"}:
        raise ValueError("Input file must be HTML.")
    if not input_html.exists():
        raise FileNotFoundError(f"Input file not found: {input_html}")

    output_path = output_html or input_html.with_name(f"{input_html.stem}.mathjax.html")
    output_path.write_text(render_mathjax_preview(input_html.read_text(encoding="utf-8")), encoding="utf-8")
    return output_path

