#!/usr/bin/env python
"""Convert the decompiled 6.7.13 API CHM (gb18030 XHTML pages) into readable
Markdown, mirrored from the CHM table of contents (API接口说明.hhc).

Usage:
    python tools/chm_to_md.py <compiled_html_dir> <out_dir>

Output layout (flat, one .md per CHM page, prefixed by its TOC path):
    <out_dir>/README.md                    the converted TOC with titles
    <out_dir>/pages/<NNN>-<slug>.md        one file per doc page
"""
from __future__ import annotations

import html as html_mod
import os
import re
import sys
from html.parser import HTMLParser

# ---------------------------------------------------------------- TOC (.hhc)


def parse_hhc(path: str):
    """Parse the CHM TOC: returns [(depth, title, local)] entries."""
    raw = open(path, "rb").read().decode("gb18030", errors="replace")
    entries = []
    # <object type="text/sitemap"> ... <param name="Name" value="..."> <param name="Local" value="...">
    for m in re.finditer(r"<object[^>]*>(.*?)</object>", raw, re.S | re.I):
        block = m.group(1)
        name = re.search(r'name="Name"\s+value="([^"]*)"', block, re.I)
        local = re.search(r'name="Local"\s+value="([^"]*)"', block, re.I)
        if name and local:
            entries.append((html_mod.unescape(name.group(1)), local.group(1)))
    return entries


# ------------------------------------------------------------- page converter

BLOCK_TAGS = {"p", "div", "h1", "h2", "h3", "h4", "h5", "h6", "li", "tr",
              "table", "ul", "ol", "pre", "blockquote", "dl", "dt", "dd"}
SKIP_TAGS = {"script", "style"}


class PageToMd(HTMLParser):
    """Minimal XHTML->Markdown for the CTP doc pages (structure is simple:
    headings, paragraphs, lists, tables, <pre> code, <a> links)."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.out = []
        self.skip = 0
        self.list_stack = []
        self.in_pre = False
        self.in_td = False
        self.cell = []
        self.row = []
        self.table = []
        self.link_href = None
        self.link_text = []
        self.first_h1 = None

    def handle_starttag(self, tag, attrs):
        if tag in SKIP_TAGS:
            self.skip += 1
            return
        if self.skip:
            return
        a = dict(attrs)
        if tag in ("h1", "h2", "h3", "h4", "h5", "h6"):
            self.out.append("\n\n")
        elif tag in ("p", "div", "blockquote"):
            self.out.append("\n\n")
        elif tag == "br":
            self.out.append("  \n")
        elif tag == "li":
            indent = "  " * max(0, len(self.list_stack) - 1)
            self.out.append("\n%s- " % indent)
        elif tag in ("ul", "ol"):
            self.list_stack.append(tag)
        elif tag == "pre":
            self.in_pre = True
            self.out.append("\n\n```\n")
        elif tag == "table":
            self.table = []
        elif tag == "tr":
            self.row = []
        elif tag in ("td", "th"):
            self.in_td = True
            self.cell = []
        elif tag == "a":
            self.link_href = a.get("href")
            self.link_text = []
        elif tag in ("b", "strong"):
            self.out.append("**")
        elif tag in ("i", "em"):
            self.out.append("*")

    def handle_endtag(self, tag):
        if tag in SKIP_TAGS:
            self.skip = max(0, self.skip - 1)
            return
        if self.skip:
            return
        if tag in ("h1", "h2", "h3", "h4", "h5", "h6"):
            self.out.append("\n")
        elif tag == "li":
            pass
        elif tag in ("ul", "ol"):
            if self.list_stack:
                self.list_stack.pop()
            self.out.append("\n")
        elif tag == "pre":
            self.in_pre = False
            self.out.append("\n```\n\n")
        elif tag == "table":
            self._flush_table()
        elif tag in ("td", "th"):
            self.in_td = False
            self.row.append("".join(self.cell).strip())
        elif tag == "tr":
            if self.row:
                self.table.append(self.row)
        elif tag == "a":
            txt = "".join(self.link_text).strip()
            if txt and self.link_href and not self.link_href.startswith("http"):
                self.out.append("[%s](%s)" % (txt, self.link_href))
            elif txt:
                self.out.append(txt)
            self.link_href = None
            self.link_text = []
        elif tag in ("b", "strong"):
            self.out.append("**")
        elif tag in ("i", "em"):
            self.out.append("*")

    def handle_data(self, data):
        if self.skip:
            return
        if self.in_td:
            self.cell.append(data)
            return
        if self.link_href is not None:
            self.link_text.append(data)
            return
        if self.in_pre:
            self.out.append(data)
            return
        self.out.append(data)

    def _flush_table(self):
        rows = [r for r in self.table if any(c for c in r)]
        self.table = []
        if not rows:
            return
        width = max(len(r) for r in rows)
        rows = [r + [""] * (width - len(r)) for r in rows]
        self.out.append("\n\n")
        self.out.append("| " + " | ".join(rows[0]) + " |\n")
        self.out.append("|" + "|".join(["---"] * width) + "|\n")
        for r in rows[1:]:
            self.out.append("| " + " | ".join(r) + " |\n")
        self.out.append("\n")

    def markdown(self) -> str:
        text = "".join(self.out)
        # collapse 3+ newlines, trim trailing spaces per line
        text = re.sub(r"[ \t]+\n", "\n", text)
        text = re.sub(r"\n{3,}", "\n\n", text)
        text = text.strip()
        # --- page chrome cleanup (CHM lesson theme artifacts) -------------
        lines = []
        for line in text.split("\n"):
            s = line.strip()
            if not s:
                lines.append("")
                continue
            # intra-page anchor-only lines (top TOC / 回到顶部 / nav footer)
            if re.fullmatch(r"\[[^\]]*\]\(#[^)]*\)", s):
                continue
            if re.match(r"^\[< 前页\]|^\[回目录\]", s):
                continue
            if re.fullmatch(r"\*{2,}", s):  # empty bold artifacts
                continue
            lines.append(line)
        text = "\n".join(lines)
        text = re.sub(r"\n{3,}", "\n\n", text)
        return text.strip()


def convert_page(path: str) -> str:
    raw = open(path, "rb").read().decode("gb18030", errors="replace")
    # the lesson content lives in <body>; drop <head> noise entirely
    i = raw.find("<body")
    if i >= 0:
        raw = raw[i:]
    p = PageToMd()
    p.feed(raw)
    return p.markdown()


# ------------------------------------------------------------------- driver


def slug(name: str) -> str:
    s = re.sub(r"[^0-9A-Za-z._-]+", "-", name)
    return s.strip("-")[:80] or "page"


def main() -> int:
    src = sys.argv[1] if len(sys.argv) > 1 else "docs/notes/assets/compiled_html"
    out = sys.argv[2] if len(sys.argv) > 2 else "docs/api-doc-md"
    os.makedirs(os.path.join(out, "pages"), exist_ok=True)

    toc = parse_hhc(os.path.join(src, "API接口说明.hhc"))
    index = ["# 《CTP 6.7.13 API接口说明》Markdown 版\n",
             "由 `tools/chm_to_md.py` 从官方 CHM 反编译页面转换（gb18030 → UTF-8）。\n",
             "源：`ctpsdk/6.7.13_20260225/docs/6.7.13_API接口说明.chm`。\n",
             "配套：SDK 错误码全集（error.xml 299 条可读表）见 [`docs/错误码全集.md`](../错误码全集.md)。\n",
             "页面为扁平化转换，原 CHM 页间相对链接不可达——导航请用本索引。\n"]
    n = 0
    for title, local in toc:
        page = os.path.join(src, local.replace("\\", "/"))
        if not os.path.isfile(page):
            index.append("- %s — **缺失** (%s)" % (title, local))
            continue
        md = convert_page(page)
        n += 1
        fname = "pages/%03d-%s.md" % (n, slug(local))
        with open(os.path.join(out, fname), "w", encoding="utf-8") as f:
            f.write("# %s\n\n" % title)
            f.write(md + "\n")
        index.append("- [%s](%s)" % (title, fname))
    with open(os.path.join(out, "README.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(index) + "\n")
    print("converted %d pages -> %s" % (n, out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
