#!/usr/bin/env python3
"""Zero-dependency HTML-to-Markdown converter using Python's stdlib html.parser."""

from __future__ import annotations

import html
import re
import urllib.parse
from html.parser import HTMLParser

SKIP_TAGS = {"script", "style", "head", "noscript", "svg", "template"}
HEADING_TAGS = {"h1": 1, "h2": 2, "h3": 3, "h4": 4, "h5": 5, "h6": 6}


class _HtmlToMarkdown(HTMLParser):
    def __init__(self, base_url: str = "") -> None:
        super().__init__(convert_charrefs=True)
        self.base_url = base_url
        self._out: list[str] = []
        self._skip_depth = 0
        self._pre_depth = 0
        self._code_depth = 0
        self._blockquote_depth = 0
        self._list_stack: list[tuple[str, int]] = []  # ("ul"|"ol", counter)
        self._link_stack: list[tuple[str, int]] = []  # (href, out_index_at_start)
        self._in_table = False
        self._table_rows: list[list[tuple[bool, str]]] = []
        self._current_row: list[tuple[bool, str]] | None = None
        self._cell_is_header = False
        self._cell_buf: list[str] | None = None
        self._dt_active = False

    def _emit(self, text: str) -> None:
        if self._cell_buf is not None:
            self._cell_buf.append(text)
        else:
            self._out.append(text)

    def _ensure_blank_line(self) -> None:
        if self._cell_buf is not None:
            return
        tail = "".join(self._out[-4:])
        if not tail:
            return
        if tail.endswith("\n\n"):
            return
        if tail.endswith("\n"):
            self._out.append("\n")
        else:
            self._out.append("\n\n")

    def _ensure_newline(self) -> None:
        if self._cell_buf is not None:
            self._cell_buf.append(" ")
            return
        tail = "".join(self._out[-2:])
        if tail and not tail.endswith("\n"):
            self._out.append("\n")

    def _flush_table(self) -> None:
        if not self._table_rows:
            return
        max_cols = max(len(r) for r in self._table_rows)
        if max_cols == 0:
            self._table_rows.clear()
            return

        def clean_cell(s: str) -> str:
            s = re.sub(r"\s+", " ", s).strip()
            return s.replace("|", "\\|")

        norm_rows = []
        for row in self._table_rows:
            cells = [clean_cell(c[1]) for c in row]
            while len(cells) < max_cols:
                cells.append("")
            norm_rows.append(cells)

        self._ensure_blank_line()
        header = norm_rows[0]
        self._out.append("| " + " | ".join(header) + " |\n")
        self._out.append("| " + " | ".join(["---"] * max_cols) + " |\n")
        for row in norm_rows[1:]:
            self._out.append("| " + " | ".join(row) + " |\n")
        self._out.append("\n")
        self._table_rows.clear()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        if tag in SKIP_TAGS:
            self._skip_depth += 1
            return
        if self._skip_depth > 0:
            return

        attr_map = dict(attrs)

        if tag in HEADING_TAGS:
            self._ensure_blank_line()
            self._emit("#" * HEADING_TAGS[tag] + " ")
        elif tag == "p":
            self._ensure_blank_line()
        elif tag == "br":
            self._emit("\n" if self._pre_depth > 0 else "  \n")
        elif tag == "hr":
            self._ensure_blank_line()
            self._emit("---\n\n")
        elif tag == "pre":
            self._ensure_blank_line()
            self._pre_depth += 1
            self._emit("```\n")
        elif tag in ("code", "tt", "kbd", "var"):
            self._code_depth += 1
            if self._pre_depth == 0:
                self._emit("`")
        elif tag in ("strong", "b"):
            if self._pre_depth == 0:
                self._emit("**")
        elif tag in ("em", "i"):
            if self._pre_depth == 0:
                self._emit("*")
        elif tag == "blockquote":
            self._ensure_blank_line()
            self._blockquote_depth += 1
            self._emit("> ")
        elif tag == "ul":
            self._ensure_newline()
            self._list_stack.append(("ul", 0))
        elif tag == "ol":
            self._ensure_newline()
            self._list_stack.append(("ol", 0))
        elif tag == "li":
            self._ensure_newline()
            indent = "  " * max(0, len(self._list_stack) - 1)
            if self._list_stack and self._list_stack[-1][0] == "ol":
                kind, count = self._list_stack[-1]
                count += 1
                self._list_stack[-1] = (kind, count)
                self._emit(f"{indent}{count}. ")
            else:
                self._emit(f"{indent}- ")
        elif tag == "dl":
            self._ensure_blank_line()
        elif tag == "dt":
            self._ensure_newline()
            self._dt_active = True
            self._emit("- **")
        elif tag == "dd":
            self._ensure_newline()
            self._emit("  ")
        elif tag == "a":
            href = (attr_map.get("href") or "").strip()
            if href and not href.startswith("javascript:"):
                if self.base_url and not href.startswith("#"):
                    href = urllib.parse.urljoin(self.base_url, href)
                buf = self._cell_buf if self._cell_buf is not None else self._out
                self._link_stack.append((href, len(buf)))
                self._emit("[")
        elif tag == "table":
            self._in_table = True
            self._table_rows = []
        elif tag == "tr" and self._in_table:
            self._current_row = []
        elif tag in ("th", "td") and self._in_table and self._current_row is not None:
            self._cell_is_header = tag == "th"
            self._cell_buf = []

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in SKIP_TAGS:
            self._skip_depth = max(0, self._skip_depth - 1)
            return
        if self._skip_depth > 0:
            return

        if tag in HEADING_TAGS:
            self._ensure_blank_line()
        elif tag == "p":
            self._ensure_blank_line()
        elif tag == "pre":
            if self._pre_depth > 0:
                self._pre_depth -= 1
                self._ensure_newline()
                self._emit("```\n\n")
        elif tag in ("code", "tt", "kbd", "var"):
            if self._code_depth > 0:
                self._code_depth -= 1
                if self._pre_depth == 0:
                    self._emit("`")
        elif tag in ("strong", "b"):
            if self._pre_depth == 0:
                self._emit("**")
        elif tag in ("em", "i"):
            if self._pre_depth == 0:
                self._emit("*")
        elif tag == "blockquote":
            self._blockquote_depth = max(0, self._blockquote_depth - 1)
            self._ensure_blank_line()
        elif tag in ("ul", "ol"):
            if self._list_stack:
                self._list_stack.pop()
            self._ensure_blank_line()
        elif tag == "li":
            self._ensure_newline()
        elif tag == "dt":
            if self._dt_active:
                self._dt_active = False
                self._emit("**")
            self._ensure_newline()
        elif tag == "dd":
            self._ensure_blank_line()
        elif tag == "a":
            if self._link_stack:
                href, start_idx = self._link_stack.pop()
                buf = self._cell_buf if self._cell_buf is not None else self._out
                label = "".join(buf[start_idx + 1 :]).strip()
                if label:
                    self._emit(f"]({href})")
                else:
                    # Drop empty anchor tags (e.g., <a name="foo"></a>)
                    del buf[start_idx:]
        elif tag in ("th", "td") and self._in_table and self._cell_buf is not None:
            text = "".join(self._cell_buf)
            self._cell_buf = None
            if self._current_row is not None:
                self._current_row.append((self._cell_is_header, text))
        elif tag == "tr" and self._in_table and self._current_row is not None:
            if any(c[1].strip() for c in self._current_row):
                self._table_rows.append(self._current_row)
            self._current_row = None
        elif tag == "table" and self._in_table:
            self._in_table = False
            self._flush_table()

    def handle_data(self, data: str) -> None:
        if self._skip_depth > 0 or not data:
            return
        if self._pre_depth > 0:
            self._emit(data)
            return
        text = re.sub(r"[ \t\r\n]+", " ", data)
        if not text.strip():
            # Keep single space between inline elements if previous token wasn't whitespace/newline
            buf = self._cell_buf if self._cell_buf is not None else self._out
            if buf and not buf[-1].endswith((" ", "\n", "[", "(", "**", "*", "`")):
                self._emit(" ")
            return
        self._emit(text)

    def markdown(self) -> str:
        raw = html.unescape("".join(self._out))
        # Clean trailing spaces on non-break lines and collapse >2 consecutive blank lines
        lines = [line.rstrip() for line in raw.splitlines()]
        cleaned = "\n".join(lines)
        cleaned = re.sub(r"\n{3,}", "\n\n", cleaned).strip()
        return cleaned + "\n"


def html_to_markdown(html_text: str, base_url: str = "") -> str:
    """Convert an HTML document string into clean GitHub-Flavored Markdown."""
    parser = _HtmlToMarkdown(base_url=base_url)
    parser.feed(html_text)
    parser.close()
    return parser.markdown()
