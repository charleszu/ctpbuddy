#!/usr/bin/env python
"""Convert the decompiled 6.7.13 API CHM (gb18030 XHTML pages) into readable
Markdown, mirrored from the CHM table of contents (API接口说明.hhc).

Usage:
    python tools/chm_to_md.py <compiled_html_dir> <out_dir>

Output layout (flat, one .md per CHM page, prefixed by its TOC path):
    <out_dir>/README.md                    the converted TOC with titles
    <out_dir>/pages/<NNN>-<slug>.md        one file per doc page

Cross-page links are rewritten at conversion time onto the flattened
filenames, so `详见[接口中一些重要序号说明](...)`-style references resolve
instead of dying on `../../QTYWGZ/*.html`. The CHM's own `anchor-id-*`
landing points are emitted verbatim (`<a id="anchor-id-NN"></a>`), keeping
`#anchor-id-NN` fragments jumpable. Targets that never made it into the TOC
are kept as-is and reported on stderr.
"""
from __future__ import annotations

import html as html_mod
import os
import posixpath
import re
import sys
import urllib.parse
from html.parser import HTMLParser
from typing import Dict, List, Optional, Set  # noqa: F401  (annotations)

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
# CHM lesson-theme chrome links. They target `_index.html` section landings /
# intra-page tops that the flat Markdown intentionally drops -- they are never
# content references, so they must not count as broken cross-page links.
CHROME_LINK_TEXTS = {"回到顶部", "回目录"}


class PageToMd(HTMLParser):
    """Minimal XHTML->Markdown for the CTP doc pages (structure is simple:
    headings, paragraphs, lists, tables, <pre> code, <a> links)."""

    def __init__(self, local: str = "", link_map: Optional[dict] = None):
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
        #: this page's own CHM-relative path (e.g. "QTYWGZ/BJHXJ.html")
        self.local = local
        #: normalized CHM path -> flattened "<NNN>-<slug>.md"
        self.link_map = link_map or {}
        #: hrefs whose target was not in the converted TOC (stay verbatim)
        self.unresolved: List[tuple] = []

    # ---- cross-page links -------------------------------------------------

    def resolve_href(self, href: str) -> Optional[str]:
        """Rewrite one CHM href onto the flat Markdown layout.

        Returns the new href (path + preserved fragment), or None when the
        href is intra-page / external / not a converted page.
        """
        path, _, frag = href.partition("#")
        if not path or path.startswith(("http://", "https://", "mailto:")):
            return None
        p = urllib.parse.unquote(path.replace("\\", "/"))
        p = posixpath.normpath(posixpath.join(posixpath.dirname(self.local), p))
        # every doc page lives under one virtual root, so a parent-relative
        # href from a root-level page (../QTYWGZ/X.html) clamps at the root
        while p.startswith("../") or p == "..":
            p = p[3:] if p.startswith("../") else ""
        hit = self.link_map.get(p.upper().lstrip("/"))
        if hit is None:
            self.unresolved.append((href, p))
            return None
        return "%s#%s" % (hit, frag) if frag else hit

    def handle_starttag(self, tag, attrs):
        if tag in SKIP_TAGS:
            self.skip += 1
            return
        if self.skip:
            return
        a = dict(attrs)
        # The CHM's own anchor landing points (`anchor-id-NN`) are what the
        # pages' `#anchor-id-NN` links target. Emit them verbatim so those
        # fragments keep resolving in the flat layout (GFM honors inline
        # HTML <a id=...>).
        aid = a.get("id") or a.get("name")
        if aid and not self.skip:
            self.out.append('<a id="%s"></a>' % html_mod.escape(aid, quote=True))
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
            is_chrome = txt in CHROME_LINK_TEXTS or txt in ("< 前页", "后页 >")
            if txt and self.link_href and not self.link_href.startswith("http"):
                if is_chrome:
                    pass  # nav chrome: dropped, and never broken-link noise
                else:
                    hit = self.resolve_href(self.link_href)
                    self.out.append("[%s](%s)" % (txt, hit if hit is not None else self.link_href))
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


def convert_page(path: str, local: str = "", link_map: Optional[Dict[str, str]] = None):
    raw = open(path, "rb").read().decode("gb18030", errors="replace")
    # the lesson content lives in <body>; drop <head> noise entirely
    i = raw.find("<body")
    if i >= 0:
        raw = raw[i:]
    p = PageToMd(local=local, link_map=link_map)
    p.feed(raw)
    return p.markdown(), p.unresolved


# ------------------------------------------------------------------- driver


def slug(name: str) -> str:
    s = re.sub(r"[^0-9A-Za-z._-]+", "-", name)
    return s.strip("-")[:80] or "page"


#: upper relative path -> actual on-disk relative path (CHM hrefs are
#: case-insensitive; .hhc keys were upper-cased into the link maps)
_SRC_INDEX: Dict[str, str] = {}


def _src_path_of(upper: str, src: str) -> str:
    if not _SRC_INDEX:
        for root, _dirs, files in os.walk(src):
            for fn in files:
                rel = os.path.relpath(os.path.join(root, fn), src).replace("\\", "/")
                _SRC_INDEX.setdefault(rel.upper(), rel)
    return _SRC_INDEX.get(upper, upper.lower())


def main() -> int:
    src = sys.argv[1] if len(sys.argv) > 1 else "docs/notes/assets/compiled_html"
    out = sys.argv[2] if len(sys.argv) > 2 else "docs/api-doc-md"
    os.makedirs(os.path.join(out, "pages"), exist_ok=True)

    def to_fname(n: int, local: str) -> str:
        return "pages/%03d-%s.md" % (n, slug(local))

    # ---- pass 1: the mirrored TOC --------------------------------------
    plan = []  # [(title, local, fname)] in README order
    link_map = {}  # upper CHM path -> fname
    counter = 0
    for title, local in parse_hhc(os.path.join(src, "API接口说明.hhc")):
        local = local.replace("\\", "/")
        if not os.path.isfile(os.path.join(src, local)):
            plan.append((title, local, None))  # listed as missing
            continue
        counter += 1
        fname = to_fname(counter, local)
        plan.append((title, local, fname))
        link_map[local.upper()] = fname

    # ---- pass 2: convert, then adopt disk-present pages the TOC omitted -
    # Some referenced pages exist in the decompiled CHM but were never listed
    # in its .hhc (section landings, orphaned API-dir copies). They are real
    # content: convert them as appendix pages (numbering after the TOC) so
    # references pointing at them resolve too. Repeat until no new page shows
    # up (an appendix may itself link further appendices).
    extras: Dict[str, str] = {}  # upper CHM path -> fname
    cache: Dict[str, tuple] = {}  # fname -> (title_or_None, md)
    misses: Set[tuple] = set()
    for _round in range(4):
        full_map = dict(link_map)
        full_map.update(extras)
        misses = set()
        cache.clear()
        for title, local, fname in plan:
            if fname is None:
                continue
            md, miss = convert_page(os.path.join(src, local), local=local, link_map=full_map)
            cache[fname] = (title, md)
            misses.update((local, h, p) for h, p in miss)
        for upper, fname in list(extras.items()):
            local = _src_path_of(upper, src)
            md, miss = convert_page(os.path.join(src, local), local=local, link_map=full_map)
            cache[fname] = (None, md)
            misses.update((local, h, p) for h, p in miss)
        fresh = sorted({p for _s, _h, p in misses
                        if p.upper() not in full_map and os.path.isfile(os.path.join(src, p))})
        if not fresh:
            break
        for p in fresh:
            counter += 1
            fname = to_fname(counter, p)
            extras[p.upper()] = fname
            plan.append((None, p, fname))

    # ---- write ------------------------------------------------------------
    # targets that are absent even from the decompiled set: already dead in
    # the official CHM itself -- kept verbatim, surfaced, never redirected
    leftover = sorted({(_h, p) for _s, _h, p in misses
                       if not os.path.isfile(os.path.join(src, p))})
    index = ["# 《CTP 6.7.13 API接口说明》Markdown 版\n",
             "由 `tools/chm_to_md.py` 从官方 CHM 反编译页面转换（gb18030 → UTF-8）。\n",
             "源：`ctpsdk/6.7.13_20260225/docs/6.7.13_API接口说明.chm`。\n",
             "配套：SDK 错误码全集（error.xml 299 条可读表）见 [`docs/错误码全集.md`](../错误码全集.md)。\n",
             "页间链接已在转换时重写到扁平文件名；CHM 的 `anchor-id-*` 锚点原样保留，`#anchor-id-NN` 可直接跳转；\n",
             ".hhc 目录未收录但被正文引用的页面见文末附录。\n"]
    wrote = 0
    appendix = []
    for title, local, fname in plan:
        if fname is None:
            index.append("- %s — **缺失** (%s)" % (title, local))
            continue
        md_title, md = cache.get(fname, (None, ""))
        if title is None:  # appendix page: take its own h1 as the title
            title = md.split("\n", 1)[0].lstrip("# ").strip() or local
            appendix.append((title, fname))
        with open(os.path.join(out, fname), "w", encoding="utf-8") as f:
            f.write("# %s\n\n" % title)
            f.write(md + "\n")
        wrote += 1
        index.append("- [%s](%s)" % (title, fname))
    if appendix:
        index.append("\n## 附录：目录未收录但被引用的页面\n")
        for title, fname in appendix:
            index.append("- [%s](%s)" % (title, fname))
    if leftover:
        by_base: Dict[str, tuple] = {}
        for title, local, fname in plan:
            if fname:
                by_base.setdefault(posixpath.basename(local).lower(), (title, fname))
        index.append("\n## 已知死链（官方 CHM 源文件即不可达）\n")
        index.append("以下引用的目标页不存在于反编译集中——官方 CHM 里点它们同样到不了。")
        index.append("按原样保留，未劫持到其他页面；括号内为同文件名的可替代页（若存在）。\n")
        for h, p in leftover:
            alt = by_base.get(posixpath.basename(p).lower())
            index.append("- `%s`%s" % (h, " — 同名可替代：[%s](%s)" % alt if alt else ""))
    with open(os.path.join(out, "README.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(index) + "\n")
    print("converted %d pages (%d appendix) -> %s" % (wrote, len(appendix), out))

    if leftover:
        # absent even from the decompile: broken in the source CHM itself
        print("still-dead links (target absent from the decompiled CHM): %d" % len(leftover),
              file=sys.stderr)
        for h, p in leftover:
            print("  %s\t(-> %s)" % (h, p), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
