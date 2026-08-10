from __future__ import annotations

from html.parser import HTMLParser


class HtmlToTypstParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self._chunks: list[str] = []
        self._list_depth = 0
        self._skip_depth = 0
        self._math_depth = 0
        self._heading_level: int | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in {"script", "style", "head"}:
            self._skip_depth += 1
            return
        if self._skip_depth:
            return

        attr_map = dict(attrs)
        classes = set((attr_map.get("class") or "").split())
        if "math" in classes or tag in {"math"}:
            self._math_depth += 1
            self._append("$ ")
            return

        if tag in {"p", "div", "section", "article", "tr"}:
            self._blank_line()
        elif tag in {"br"}:
            self._append("\n")
        elif tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self._blank_line()
            self._heading_level = int(tag[1])
            self._append("=" * self._heading_level + " ")
        elif tag in {"strong", "b"}:
            self._append("*")
        elif tag in {"em", "i"}:
            self._append("_")
        elif tag == "ul":
            self._list_depth += 1
            self._blank_line()
        elif tag == "li":
            self._append("\n" + "  " * max(self._list_depth - 1, 0) + "- ")
        elif tag in {"td", "th"}:
            self._append(" | ")

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style", "head"} and self._skip_depth:
            self._skip_depth -= 1
            return
        if self._skip_depth:
            return

        if self._math_depth and tag in {"span", "div", "math"}:
            self._append(" $")
            self._math_depth -= 1
            return

        if tag in {"p", "div", "section", "article", "tr", "table"}:
            self._blank_line()
        elif tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self._heading_level = None
            self._blank_line()
        elif tag in {"strong", "b"}:
            self._append("*")
        elif tag in {"em", "i"}:
            self._append("_")
        elif tag == "ul":
            self._list_depth = max(self._list_depth - 1, 0)
            self._blank_line()

    def handle_data(self, data: str) -> None:
        if self._skip_depth:
            return
        text = " ".join(data.split())
        if not text:
            return
        if self._math_depth:
            self._append(text)
        else:
            self._append(_escape_typst_text(text))

    def render(self) -> str:
        text = "".join(self._chunks)
        lines = [line.rstrip() for line in text.splitlines()]
        compact: list[str] = []
        previous_blank = True
        for line in lines:
            blank = not line.strip()
            if blank and previous_blank:
                continue
            compact.append(line)
            previous_blank = blank
        return "#set page(margin: 1in)\n#set text(size: 11pt)\n\n" + "\n".join(compact).strip() + "\n"

    def _append(self, text: str) -> None:
        if self._chunks and self._chunks[-1] and not self._chunks[-1].endswith(("\n", " ", "| ")):
            if text and not text.startswith(("\n", " ", ".", ",", ":", ";", ")", "]", "}")):
                self._chunks.append(" ")
        self._chunks.append(text)

    def _blank_line(self) -> None:
        if not self._chunks:
            return
        current = "".join(self._chunks)
        if not current.endswith("\n\n"):
            self._chunks.append("\n\n")


def html_to_typst(html: str) -> str:
    parser = HtmlToTypstParser()
    parser.feed(html)
    parser.close()
    return parser.render()


def _escape_typst_text(text: str) -> str:
    return (
        text.replace("\\", "\\\\")
        .replace("#", "\\#")
        .replace("$", "\\$")
        .replace("*", "\\*")
        .replace("_", "\\_")
        .replace("[", "\\[")
        .replace("]", "\\]")
    )

