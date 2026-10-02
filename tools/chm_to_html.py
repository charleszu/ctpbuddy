#!/usr/bin/env python
"""Convert the decompiled 6.7.13 API CHH (gb18030 XHTML pages) into clean,
self-contained HTML, mirrored from the CHM TOC (API接口说明.hhc).

Usage:
    python tools/chm_to_html.py <compiled_html_dir> <out_dir>

Output layout (flat, one .html per CHM page, prefixed by its TOC path):
    <out_dir>/index.html                 the converted TOC with titles
    <out_dir>/pages/<NNN>-<slug>.html    one file per doc page

Why HTML instead of Markdown: the decompiled pages carry a CHH "lesson
theme" chrome (left-menu, back-to-top, foot nav, theme switcher, script
imports, anchor-id landers) plus true content markup (tables, <pre>/<code>,
data-URI images, legacy <font color>). Flattening to Markdown both loses the
rich bits and leaves the chrome behind as noise. Here the chrome is stripped
structurally and the content is preserved as real HTML.

Conversion-time rewrites:
- cross-page links are rewritten onto the flat filenames, so
  `详见[接口中一些重要序号说明](...)`-style references resolve instead of
  dying on `../../QTYWGZ/*.html`; intra-page fragment links land on the
  `anchor-id-*` points that were real CHM TOC targets;
- the per-page left-menu anchors (random UUID ids) are dropped together with
  the menu that referenced them -- nothing else points at them;
- CHM <error id=... value=... prompt=.../> markers (error.xml named codes)
  render as readable inline chips;
- targets that never made it into the TOC are appended as appendix pages
  (numbering after the TOC) so references pointing at them resolve too;
- links whose target is absent even from the decompiled set were already
  dead in the official CHH -- kept verbatim, reported on stderr and
  disclosed in index.html, never silently redirected.
"""
from __future__ import annotations

import html as html_mod
import os
import posixpath
import re
import shutil
import sys
import urllib.parse
from html.parser import HTMLParser
from typing import Dict, List, Optional, Set, Tuple

# ---------------------------------------------------------------- TOC (.hhc)


def parse_hhc(path: str) -> List[Tuple[str, str]]:
    """Parse the CHM TOC: returns [(title, local)] entries."""
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

#: HTML void elements (never pushed on the open-tag stack)
VOID_TAGS = {"br", "hr", "img", "meta", "link", "input", "base", "col",
             "param", "area", "source", "wbr"}

#: stack marker for an element inside a dropped chrome subtree
SKIPPED = "\x00skip"

#: wrappers we render transparently (they exist only in the CHM theme)
TRANSPARENT_DIV_IDS = {"printArea", "content"}

#: div/class subtrees that are pure theme chrome -- dropped wholesale
DROP_CLASSES = {"left_menu", "left_menu_content", "left_menu_title",
                "back_to_top_link", "foot", "foot-comment-area"}
#: elements identified by id that are pure chrome
DROP_IDS = {"left_menu", "theme_switcher", "author"}
#: <hr class=...> chrome rules (content <hr>s are kept)
DROP_HR_CLASSES = {"headerline", "footline"}

#: attributes worth keeping per tag (everything else is theme noise)
KEEP_ATTRS = {
    "a": {"href"},
    "img": {"src", "alt", "width", "height"},
}
#: inline style props worth keeping (table cell alignment)
KEEP_STYLE_PROPS = {"text-align", "text-indent", "vertical-align", "color",
                    "background-color", "font-weight", "font-style",
                    "text-decoration"}


def _clean_attrs(tag: str, attrs: Dict[str, Optional[str]]) -> Dict[str, str]:
    keep = KEEP_ATTRS.get(tag)
    if keep is None:
        return {}
    out = {}
    for k, v in attrs.items():
        if k in keep and v not in (None, ""):
            out[k] = v
    if tag in ("td", "th", "p", "span"):
        style = attrs.get("style", "")
        if style:
            props = []
            for part in style.split(";"):
                if ":" not in part:
                    continue
                k, _, v = part.partition(":")
                k = k.strip().lower()
                v = v.strip()
                if k in KEEP_STYLE_PROPS and v:
                    props.append("%s: %s" % (k, v))
            if props:
                out["style"] = "; ".join(props)
    return out


def _emit_attrs(attrs: Dict[str, str]) -> str:
    if not attrs:
        return ""
    return " " + " ".join('%s="%s"' % (k, html_mod.escape(v, quote=True))
                          for k, v in attrs.items())


class PageToHtml(HTMLParser):
    """XHTML -> clean HTML for one decompiled CTP doc page.

    Tracks an open-tag stack so whole chrome subtrees (left menu, foot nav)
    can be skipped by depth, and drops per-element noise (classes, theme
    ids, onclick handlers, stylesheet imports) from what survives.
    """

    def __init__(self, local: str = "", link_map: Optional[dict] = None,
                 keep_anchors: Optional[Set[str]] = None,
                 self_fname: str = ""):
        super().__init__(convert_charrefs=True)
        self.out: List[str] = []
        self.local = local
        self.link_map = link_map or {}
        #: this page's own flat name, for intra-page fragment bookkeeping
        self.self_fname = self_fname
        #: (target flat name, fragment) for every content link with a frag
        self.live_frags: Set[Tuple[str, str]] = set()
        #: fragment ids on this page that something still references
        self.keep_anchors = keep_anchors or set()
        self.unresolved: List[Tuple[str, str]] = []
        #: anchor ids actually emitted (for the dangling-fragment audit)
        self.emitted_anchors: Set[str] = set()
        #: parallel stack: tag -> depth of "skipping" state when opened
        self.stack: List[Tuple[str, Optional[str]]] = []
        self.skip_depth: Optional[int] = None
        self.skip_nesting = 0

    # ---- subtree skipping -----------------------------------------------

    def _skipping(self, tag: str, attrs: Dict[str, Optional[str]]) -> bool:
        """Decide whether this element opens a chrome subtree to skip."""
        cls = set((attrs.get("class") or "").split())
        eid = attrs.get("id") or ""
        if tag in ("script", "style", "head"):
            return True
        if cls & DROP_CLASSES or eid in DROP_IDS:
            return True
        if tag == "hr" and cls & DROP_HR_CLASSES:
            return True
        if tag == "img" and eid == "theme_switcher":
            return True
        if tag == "p" and cls & {"fileheader", "back_to_top_link"}:
            # <p class=fileheader> is the page title; emitted separately
            return True
        return False

    def _transparent(self, tag: str, attrs: Dict[str, Optional[str]]) -> bool:
        return tag == "div" and (attrs.get("id") or "") in TRANSPARENT_DIV_IDS

    # ---- cross-page links ------------------------------------------------

    def _split_href(self, href: str) -> Tuple[Optional[str], str]:
        """Split a raw href into (resolved flat target or None, fragment).

        None means: intra-page/external/unknown -> caller decides.
        """
        path, _, frag = href.partition("#")
        if not path or path.startswith(("http://", "https://", "mailto:", "data:")):
            return None, frag
        p = urllib.parse.unquote(path.replace("\\", "/"))
        p = posixpath.normpath(posixpath.join(posixpath.dirname(self.local), p))
        # every doc page lives under one virtual root, so a parent-relative
        # href from a root-level page (../QTYWGZ/X.html) clamps at the root
        while p.startswith("../") or p == "..":
            p = p[3:] if p.startswith("../") else ""
        hit = self.link_map.get(p.upper().lstrip("/"))
        if hit is None:
            self.unresolved.append((href, p))
            return None, frag
        return hit, frag

    def _render_href(self, href: str) -> Optional[str]:
        """Final href for an <a>: rewritten flat path (+frag), '#frag',
        verbatim external URL, or None when the link should be dropped
        (chrome nav like 回到顶部, or a fragment nothing anchors anymore).

        Every fragment that a *content* link points at is recorded, even
        when it turns out to be dangling -- the page-local menu never gets
        here (it is dropped as chrome), so the record is free of the theme
        UUID noise.
        """
        path, frag = href.partition("#")[0], href.partition("#")[2]
        if path.startswith(("http://", "https://", "mailto:")):
            return href
        if not path:
            # intra-page fragment: keep only real CHM anchors
            if frag:
                self.live_frags.add((self.self_fname, frag))
            if frag and (frag in self.keep_anchors or frag.startswith("anchor-id")):
                return "#" + frag
            return None
        hit, _ = self._split_href(href)
        if hit is None:
            return None
        if frag:
            self.live_frags.add((hit, frag))
        return "%s#%s" % (hit, frag) if frag else hit

    # ---- parser callbacks -------------------------------------------------

    def handle_starttag(self, tag: str, attrs_list):
        attrs = {k: (v if v is not None else "") for k, v in attrs_list}
        # --- inside a chrome subtree: count nesting, emit nothing ---------
        if self.skip_depth is not None:
            if tag not in VOID_TAGS:
                self.stack.append((tag, SKIPPED))
                self.skip_nesting += 1
            return
        if self._skipping(tag, attrs):
            if tag in VOID_TAGS:
                return  # a chrome <hr>/<img>: dropped, never opens a subtree
            self.skip_depth = len(self.stack)
            self.skip_nesting = 1
            self.stack.append((tag, SKIPPED))
            return
        # <error id=... value=... prompt=.../> : error.xml named code chip
        if tag == "error":
            self.out.append(_error_chip(attrs))
            return
        if tag in VOID_TAGS:
            self._emit_void(tag, attrs)
            return
        if self._transparent(tag, attrs):
            self.stack.append((tag, None))  # rendered as nothing
            return
        # legacy <font color=...> -> <span style="color:...">
        if tag == "font":
            style = self._font_style(attrs)
            self.out.append("<span%s>" % _emit_attrs({"style": style} if style else {}))
            self.stack.append((tag, None))
            return
        if tag == "a":
            href = self._render_href(attrs.get("href", ""))
            if href is None:
                self.stack.append((tag, "drop-link"))
                return
            self.out.append("<a%s>" % _emit_attrs({"href": href}))
            self.stack.append((tag, None))
            return
        # anchor landing points: keep only when still referenced
        aid = attrs.get("id") or attrs.get("name") or ""
        if aid and tag == "span" and not self._has_content_class(attrs):
            if aid.startswith("anchor-id") or aid in self.keep_anchors:
                self.out.append('<a id="%s"></a>' % html_mod.escape(aid, quote=True))
                self.emitted_anchors.add(aid)
            if self._is_bare_anchor_span(attrs):
                self.stack.append((tag, "drop-empty"))
                return
        self.out.append("<%s%s>" % (tag, _emit_attrs(_clean_attrs(tag, attrs))))
        self.stack.append((tag, None))

    @staticmethod
    def _has_content_class(attrs: Dict[str, Optional[str]]) -> bool:
        return bool((attrs.get("class") or "").strip())

    @staticmethod
    def _is_bare_anchor_span(attrs: Dict[str, Optional[str]]) -> bool:
        """A theme anchor span: id/name + class=anchor, no other content."""
        return bool(attrs.get("id") or attrs.get("name"))

    @staticmethod
    def _font_style(attrs: Dict[str, Optional[str]]) -> str:
        props = []
        color = attrs.get("color")
        if color:
            props.append("color: %s" % color)
        # <font size=/face=> have no sane flat-layout equivalent; CSS rules
        return "; ".join(props)

    def _emit_void(self, tag: str, attrs: Dict[str, Optional[str]]):
        if tag == "hr":
            self.out.append("<hr>")
            return
        if tag == "br":
            self.out.append("<br>")
            return
        if tag == "img":
            src = attrs.get("src") or ""
            if not src or "theme_switcher" in src:
                return
            keep = {"src": src}
            alt = attrs.get("alt")
            if alt:
                keep["alt"] = alt
            self.out.append("<img%s>" % _emit_attrs(keep))
            return
        # other void tags carry no flat-layout meaning
        return

    def handle_endtag(self, tag: str):
        if tag in VOID_TAGS:
            return
        if self.skip_depth is not None:
            found, mode = self._pop_to(tag)
            if found and mode == SKIPPED:
                self.skip_nesting -= 1
                if self.skip_nesting <= 0:
                    self.skip_depth = None
            return
        if tag == "font":
            found, _mode = self._pop_to(tag)
            if found:
                self.out.append("</span>")
            return
        found, mode = self._pop_to(tag)
        if not found or mode in ("drop-link", "drop-empty"):
            return
        self.out.append("</%s>" % tag)

    def _pop_to(self, tag: str) -> Tuple[bool, Optional[str]]:
        """Pop the stack down to and including the nearest matching open
        tag. Returns (found, mode). Unclosed children are dropped
        implicitly: the decompiled chrome nests cleanly in practice, and a
        stray end tag must never eat the rest of the page.
        """
        for i in range(len(self.stack) - 1, -1, -1):
            if self.stack[i][0] == tag:
                mode = self.stack[i][1]
                del self.stack[i:]
                return True, mode
        return False, None

    def handle_data(self, data: str):
        if self.skip_depth is not None:
            return
        self.out.append(data)

    def handle_comment(self, data: str):
        return  # decompiler/IE conditional comments are noise

    def handle_decl(self, decl: str):
        return

    def unknown_decl(self, data: str):
        return

    def html(self) -> str:
        text = "".join(self.out)
        # close anything left open (defensive; sources are well-formed)
        for t, mode in reversed(self.stack):
            if mode is None and t == "font":
                text += "</span>"
        # the theme's bare <span class="anchor"></span> landers leave empty
        # spans behind once their id is dropped -- they render as nothing
        text = re.sub(r"<span>\s*</span>", "", text)
        text = re.sub(r"[ \t]+\n", "\n", text)
        text = re.sub(r"\n{3,}", "\n\n", text)
        return text.strip()


def _error_chip(attrs: Dict[str, Optional[str]]) -> str:
    """Render the CHM <error id value prompt/> marker as readable HTML."""
    name = attrs.get("id") or ""
    value = attrs.get("value") or ""
    prompt = attrs.get("prompt") or ""
    if not name:
        return ""
    body = '<code>%s</code>' % html_mod.escape(name)
    tail = []
    if value:
        tail.append(html_mod.escape(value))
    if prompt:
        tail.append(html_mod.escape(prompt))
    if tail:
        body += " — " + "：".join(tail)
    return body


_SCRIPT_RE = re.compile(r"<script\b[^>]*>.*?</script>", re.S | re.I)


def _page_body_source(path: str) -> Tuple[str, str, str]:
    """Read one decompiled page -> (raw_source, local_path, page_title).

    Only the printArea envelope (fileheader + content) is returned; the
    head, theme scripts, left menu and foot nav are left for the parser's
    chrome rules to drop.
    """
    raw = open(path, "rb").read().decode("gb18030", errors="replace")
    raw = _SCRIPT_RE.sub("", raw)

    def _tag_start(marker: str, before: int) -> int:
        """Back up from an attribute match to its opening '<'."""
        j = raw.find(marker)
        return raw.rfind("<", 0, j) if j >= 0 else before

    i = _tag_start('id="printArea"', -1)
    if i < 0:
        i = _tag_start("class='fileheader'", -1)
    if i < 0:
        i = raw.find("<body")
    seg = raw[i:] if i >= 0 else raw
    j = seg.find('<div class="foot-comment-area"')
    if j >= 0:
        seg = seg[:j]
    m = re.search(r"class=['\"]fileheader['\"][^>]*>(.*?)</p>", seg, re.S)
    title = html_mod.unescape(m.group(1)).strip() if m else ""
    return seg, raw, title


def convert_page(path: str, local: str = "",
                 link_map: Optional[Dict[str, str]] = None,
                 keep_anchors: Optional[Set[str]] = None,
                 self_fname: str = ""):
    seg, _raw, title = _page_body_source(path)
    p = PageToHtml(local=local, link_map=link_map, keep_anchors=keep_anchors,
                   self_fname=self_fname)
    p.feed(seg)
    return p.html(), p.unresolved, title, p.emitted_anchors, p.live_frags


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


_HREF_RE = re.compile(r'href\s*=\s*["\']([^"\']+)["\']', re.I)
_ID_RE = re.compile(r'\b(?:id|name)\s*=\s*["\']([^"\']+)["\']', re.I)


def _collect_fragment_refs(local: str, raw: str, link_map: Dict[str, str],
                           refs: Dict[str, Set[str]]) -> None:
    """Record which fragment ids each target page is referenced by."""
    for m in _HREF_RE.finditer(raw):
        href = urllib.parse.unquote(m.group(1).replace("\\", "/"))
        path, _, frag = href.partition("#")
        if not frag:
            continue
        if not path:
            # intra-page: the target is this page itself
            if frag.startswith("anchor-id"):
                refs.setdefault(local, set()).add(frag)
            continue
        if path.startswith(("http://", "https://", "mailto:")):
            continue
        p = posixpath.normpath(posixpath.join(posixpath.dirname(local), path))
        while p.startswith("../") or p == "..":
            p = p[3:] if p.startswith("../") else ""
        hit = link_map.get(p.upper().lstrip("/"))
        if hit and frag:
            refs.setdefault(hit, set()).add(frag)


PAGE_CSS = """\
:root { color-scheme: light; }
* { box-sizing: border-box; }
body { margin: 0; background: #fff; color: #1f2328;
       font: 16px/1.75 -apple-system, "Segoe UI", "PingFang SC",
       "Hiragino Sans GB", "Microsoft YaHei", sans-serif; }
.doc { max-width: 920px; margin: 0 auto; padding: 32px 28px 48px; }
.doc > h1.pagetitle { font-size: 1.6em; margin: 0 0 .6em;
       padding-bottom: .3em; border-bottom: 2px solid #d0d7de; }
h1 { font-size: 1.35em; margin: 1.4em 0 .5em; }
h2 { font-size: 1.2em; margin: 1.3em 0 .5em; }
h3 { font-size: 1.08em; margin: 1.2em 0 .5em; }
p { margin: .55em 0; }
a { color: #0969da; text-decoration: none; }
a:hover { text-decoration: underline; }
table { border-collapse: collapse; margin: .8em 0; font-size: .95em; }
th, td { border: 1px solid #d0d7de; padding: 5px 10px; text-align: left; }
th { background: #f6f8fa; }
pre { background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 6px;
      padding: 12px 14px; overflow: auto; font-size: .9em; line-height: 1.5; }
code { font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
       background: #f6f8fa; border-radius: 4px; padding: 0 4px; font-size: .92em; }
pre code { background: none; padding: 0; }
blockquote { margin: .6em 0; padding: .2em 1em; color: #57606a;
       border-left: 4px solid #d0d7de; }
img { max-width: 100%; height: auto; }
hr { border: none; border-top: 1px solid #d0d7de; margin: 1.6em 0; }
ul, ol { padding-left: 1.6em; }
.pager { display: flex; gap: 1.2em; max-width: 920px; margin: 0 auto;
       padding: 0 28px 40px; border-top: 1px solid #d0d7de; padding-top: 14px;
       font-size: .95em; }
.pager .home { margin-left: auto; }
"""

INDEX_CSS = """\
:root { color-scheme: light; }
* { box-sizing: border-box; }
body { margin: 0; background: #fff; color: #1f2328;
       font: 15px/1.7 -apple-system, "Segoe UI", "PingFang SC",
       "Hiragino Sans GB", "Microsoft YaHei", sans-serif; }
.wrap { max-width: 960px; margin: 0 auto; padding: 32px 24px 48px; }
h1 { font-size: 1.5em; border-bottom: 2px solid #d0d7de; padding-bottom: .3em; }
h2 { font-size: 1.15em; margin-top: 1.8em; }
a { color: #0969da; text-decoration: none; }
a:hover { text-decoration: underline; }
.toc { columns: 2; column-gap: 40px; list-style: none; padding: 0; }
.toc li { margin: 2px 0; break-inside: avoid; }
.muted { color: #57606a; }
code { font-family: ui-monospace, Consolas, monospace; background: #f6f8fa;
       border-radius: 4px; padding: 0 4px; font-size: .92em; }
@media (max-width: 700px) { .toc { columns: 1; } }
"""


def _page_doc(title: str, body: str, prev: Optional[str], nxt: Optional[str],
              first: str = "../index.html") -> str:
    def link(href: Optional[str], label: str) -> str:
        return ('<a href="%s">%s</a>' % (href, label)) if href else "<span class='muted'>%s</span>" % label
    pager = ('<nav class="pager">%s%s%s</nav>'
             % (link(prev, "← 前页"), link(first or "index.html", "目录"), link(nxt, "后页 →")))
    return ("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n"
            "<meta charset=\"utf-8\">\n"
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n"
            "<title>%s</title>\n<style>%s</style>\n</head>\n<body>\n"
            "<article class=\"doc\">\n%s\n</article>\n%s\n</body>\n</html>\n"
            % (html_mod.escape(title), PAGE_CSS, body, pager))


def _index_doc(lines: List[str]) -> str:
    return ("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n"
            "<meta charset=\"utf-8\">\n"
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n"
            "<title>《CTP 6.7.13 API接口说明》HTML 版</title>\n"
            "<style>%s</style>\n</head>\n<body>\n<div class=\"wrap\">\n%s\n</div>\n</body>\n</html>\n"
            % (INDEX_CSS, "\n".join(lines)))


def main() -> int:
    src = sys.argv[1] if len(sys.argv) > 1 else "docs/notes/assets/compiled_html"
    out = sys.argv[2] if len(sys.argv) > 2 else "docs/api-doc-html"

    def is_page(local: str) -> bool:
        return local.lower().endswith((".html", ".htm"))

    def to_fname(n: int, local: str) -> str:
        """Flat page name. Links between pages are written as bare
        filenames (both ends live in pages/), so no prefix is needed."""
        stem = re.sub(r"\.[A-Za-z0-9]{1,5}$", "", local)
        return "%03d-%s.html" % (n, slug(stem))

    # ---- pass 1: the mirrored TOC --------------------------------------
    # plan entries: [title|None, local, fname|None, is_asset]
    plan: List[List[object]] = []
    link_map: Dict[str, str] = {}   # upper CHM path -> href used from pages/
    counter = 0
    for title, local in parse_hhc(os.path.join(src, "API接口说明.hhc")):
        local = local.replace("\\", "/")
        if not os.path.isfile(os.path.join(src, local)):
            plan.append([title, local, None, False])  # listed but absent
            continue
        counter += 1
        fname = to_fname(counter, local)
        plan.append([title, local, fname, False])
        link_map[local.upper()] = fname

    # ---- pass 2: link discovery ----------------------------------------
    # Convert repeatedly until the link graph stops growing: pages the
    # .hhc never listed but that the body really references are adopted as
    # appendix pages (numbered after the TOC); real non-page assets (SDK
    # headers, error.xml, the two PDFs) are copied to files/ instead.
    assets: Dict[str, str] = {}    # upper CHM path -> href from pages/
    misses: Set[Tuple[str, str]] = set()
    live_frags: Set[Tuple[str, str]] = set()  # (target fname, fragment)
    for _round in range(5):
        misses = set()
        for entry in list(plan):
            title, local, fname, is_asset = entry
            if fname is None or is_asset:
                continue
            _b, miss, _t, _a, _lf = convert_page(
                os.path.join(src, local), local=local, link_map=link_map,
                self_fname=str(fname))
            misses.update(miss)
            live_frags.update(_lf)
        fresh = sorted({p for _h, p in misses
                        if p.upper() not in link_map and os.path.isfile(os.path.join(src, p))})
        if not fresh:
            break
        for p in fresh:
            if is_page(p):
                counter += 1
                fname = to_fname(counter, p)
            else:
                fname = "../files/%s" % posixpath.basename(p)
                assets[p.upper()] = fname
            link_map[p.upper()] = fname
            plan.append([None, p, fname, not is_page(p)])

    # ---- pass 3: which anchors are still referenced? --------------------
    # The page-local menu (random UUID fragments) is chrome and never
    # reaches the link renderer, so `live_frags` holds only fragments that
    # real content links to. Intersect those with the anchor ids each page
    # actually defines: what survives is worth emitting, what does not is
    # a dangling fragment in the official CHM too (disclosed in index).
    full_map = dict(link_map)
    keep_by_fname: Dict[str, Set[str]] = {str(e[2]): set() for e in plan
                                          if e[2] and not e[3]}
    ids_by_fname: Dict[str, Set[str]] = {}
    for title, local, fname, is_asset in plan:
        if fname is None or is_asset:
            continue
        raw = open(os.path.join(src, local), "rb").read().decode("gb18030", errors="replace")
        ids_by_fname[str(fname)] = set(_ID_RE.findall(raw))
    for tgt, frag in sorted(live_frags):
        if frag in ids_by_fname.get(tgt, set()):
            keep_by_fname.setdefault(tgt, set()).add(frag)
    # a re-run can drop ids emitted in a previous round; recompute cleanly
    keep_by_fname = {f: (frags & ids_by_fname.get(f, set()))
                     for f, frags in keep_by_fname.items()}

    # ---- write ------------------------------------------------------------
    # targets absent even from the decompiled set: already dead in the
    # official CHM itself -- kept verbatim, surfaced, never redirected
    leftover = sorted({(h, p) for h, p in misses
                       if not os.path.isfile(os.path.join(src, p))})
    # fragments a content link targets but the target page never defines
    dangling_frags = sorted({(t, f) for t, f in live_frags
                             if f not in ids_by_fname.get(t, set())})
    os.makedirs(os.path.join(out, "pages"), exist_ok=True)
    os.makedirs(os.path.join(out, "files"), exist_ok=True)
    wrote = 0
    appendix: List[Tuple[str, str]] = []
    order = [str(e[2]) for e in plan if e[2] and not e[3]]
    page_no = -1
    for title, local, fname, is_asset in plan:
        if fname is None:
            continue
        if is_asset:
            # fname is a href relative to pages/ ("../files/x.pdf")
            shutil.copyfile(os.path.join(src, local),
                            os.path.join(out, "files", posixpath.basename(str(fname))))
            continue
        page_no += 1
        body, _miss, fhtitle, emitted, _lf = convert_page(
            os.path.join(src, local), local=local, link_map=full_map,
            keep_anchors=keep_by_fname.get(str(fname), set()),
            self_fname=str(fname))
        del emitted  # kept for the audit below via ids_by_fname
        page_title = title or fhtitle or local
        if title is None:  # appendix page: its own fileheader is the title
            page_title = fhtitle or local
            appendix.append((page_title, str(fname)))
        if fhtitle:
            body = '<h1 class="pagetitle">%s</h1>\n%s' % (html_mod.escape(fhtitle), body)
        prev = order[page_no - 1] if page_no > 0 else None
        nxt = order[page_no + 1] if page_no + 1 < len(order) else None
        with open(os.path.join(out, "pages", str(fname)), "w", encoding="utf-8") as f:
            f.write(_page_doc(page_title, body, prev, nxt))
        wrote += 1

    # ---- index -----------------------------------------------------------
    head = ["<h1>《CTP 6.7.13 API接口说明》HTML 版</h1>",
            "<p>由 <code>tools/chm_to_html.py</code> 从官方 CHM 反编译页面转换"
            "（gb18030 → UTF-8）：剔除 CHM 主题框架（左侧目录、回顶部、页脚导航、"
            "主题脚本与切换按钮），保留正文结构、表格、代码块与内嵌图片。</p>",
            "<p>源：<code>ctpsdk/6.7.13_20260225/docs/6.7.13_API接口说明.chm</code>。"
            "配套：SDK 错误码全集（error.xml 299 条可读表）见 "
            "<a href=\"../错误码全集.md\"><code>docs/错误码全集.md</code></a>。</p>",
            "<p>页间链接已在转换时重写到扁平文件名；CHM 的 <code>anchor-id-*</code> "
            "锚点原样保留，<code>#anchor-id-NN</code> 可直接跳转；正文引用但 .hhc "
            "目录未收录的页面见文末附录；官方 CHM 自身不可达的链接集中公示于末节。</p>"]
    items = []
    for title, local, fname, is_asset in plan:
        if fname is None:
            items.append("<li class='muted'>%s — <strong>缺失</strong> (<code>%s</code>)</li>"
                         % (html_mod.escape(str(title or local)), html_mod.escape(local)))
        elif is_asset:
            # pages/ hrefs carry "../files/..."; the index sits one level up
            items.append("<li><a href=\"files/%s\"><code>%s</code></a> <span class='muted'>（原样附带的官方文件）</span></li>"
                         % (posixpath.basename(str(fname)), html_mod.escape(posixpath.basename(local))))
        else:
            label = title
            if label is None:
                label = next((t for t, f in appendix if f == fname), local)
            items.append("<li><a href=\"pages/%s\">%s</a></li>"
                         % (fname, html_mod.escape(str(label))))
    blocks = ["\n".join(head),
              "<h2>目录（%d 页）</h2>" % wrote,
              "<ul class=\"toc\">", "\n".join(items), "</ul>"]
    if appendix:
        blocks.append("<h2>附录：目录未收录但被正文引用的页面</h2>")
        blocks.append("<ul>")
        for title, fname in appendix:
            blocks.append("<li><a href=\"pages/%s\">%s</a></li>" % (fname, html_mod.escape(title)))
        blocks.append("</ul>")
    if leftover:
        by_base: Dict[str, Tuple[Optional[str], str]] = {}
        for title, local, fname, is_asset in plan:
            if fname and not is_asset:
                by_base.setdefault(posixpath.basename(local).lower(), (title, str(fname)))
        blocks.append("<h2>已知死链（官方 CHM 源文件即不可达）</h2>")
        blocks.append("<p>以下引用的目标页不存在于反编译集中——官方 CHM 里点它们同样到不了。"
                      "按原样保留，未劫持到其他页面；破折号后为同文件名的可替代页（若存在）。</p>")
        blocks.append("<ul>")
        for h, p in leftover:
            alt = by_base.get(posixpath.basename(p).lower())
            alt_html = (" — 同名可替代：<a href=\"pages/%s\">%s</a>"
                        % (alt[1], html_mod.escape(str(alt[0] or alt[1]))) if alt else "")
            blocks.append("<li><code>%s</code>%s</li>" % (html_mod.escape(h), alt_html))
        blocks.append("</ul>")
    if dangling_frags:
        blocks.append("<h2>悬空锚点（官方 CHM 源页即无该锚点）</h2>")
        blocks.append("<p>下列页内/页间链接带有 <code>#anchor-id-*</code> 片段，"
                      "但目标页在官方 CHM 里就没有对应锚点，点击不会跳转。链接按原样保留。</p>")
        blocks.append("<ul>")
        for fname, frag in dangling_frags:
            blocks.append("<li><a href=\"pages/%s\">%s</a> — <code>#%s</code></li>"
                          % (fname, html_mod.escape(fname), html_mod.escape(frag)))
        blocks.append("</ul>")
    with open(os.path.join(out, "index.html"), "w", encoding="utf-8") as f:
        f.write(_index_doc(blocks))
    print("converted %d pages (%d appendix, %d files, %d dangling frags) -> %s"
          % (wrote, len(appendix), len(assets), len(dangling_frags), out))

    if leftover:
        # absent even from the decompile: broken in the source CHM itself
        print("still-dead links (target absent from the decompiled CHM): %d" % len(leftover),
              file=sys.stderr)
        for h, p in leftover:
            print("  %s\t(-> %s)" % (h, p), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
