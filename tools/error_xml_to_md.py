#!/usr/bin/env python
"""Decode the CTP SDK error.xml (GB2312) into a readable UTF-8 Markdown table.

Usage:
    python tools/error_xml_to_md.py <error.xml> <out.md> [--title TITLE]

The SDK ships one error.xml per API (td/md) plus a TGate variant inside the
API doc CHM; the sets are near-identical. This tool decodes any of them.
"""
from __future__ import annotations

import os
import re
import sys
import xml.etree.ElementTree as ET


def decode_xml(path: str):
    raw = open(path, "rb").read()
    # error.xml declares gb2312; ElementTree chokes on the declaration, so
    # decode manually and strip it.
    text = raw.decode("gb18030", errors="replace")
    text = re.sub(r"^\s*<\?xml[^>]*\?>", "", text)
    text = re.sub(r"<!DOCTYPE[^>]*>", "", text)
    root = ET.fromstring(text)
    out = []
    for e in root.findall("error"):
        out.append((e.get("id"), e.get("value"), e.get("prompt", "")))
    return out


def main() -> int:
    src = sys.argv[1]
    dst = sys.argv[2]
    title = sys.argv[3] if len(sys.argv) > 3 else "CTP 错误码全集（error.xml）"
    errors = decode_xml(src)
    by_name = {e[0]: e for e in errors}
    lines = [
        "# %s" % title,
        "",
        "来源：`%s`（%d 条，GB2312 → UTF-8，prompt 原文照录）。" % (
            os.path.relpath(src, os.path.dirname(dst)).replace("\\", "/"), len(errors)),
        "",
        "CTP 客户端必须处理的全集——CTPBuddy 对每条都要有对应实现（错误码 / 文案 / 推送面）。",
        "「CTPBuddy」列标注当前实现状态（见对账章节）。",
        "",
        "| value | id | prompt（官方原文） | CTPBuddy |",
        "|---|---|---|---|",
    ]
    for name, value, prompt in errors:
        lines.append("| %s | %s | %s |  |" % (value, name, prompt))
    lines += [
        "",
        "## 备注",
        "",
        "- `NONE`（0）= 「CTP:正确」，非错误；",
        "- 同一错误码在不同 API（td/md）下触发条件不同，prompt 相同；",
        "- 负数错误码为 API 本地校验（-1 网络故障 / -2 未处理请求超过许可数 / -3 每秒发送请求数超过许可数 …）照录自 error.xml 与 API 文档。",
    ]
    with open(dst, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print("decoded %d errors -> %s" % (len(errors), dst))
    return 0


if __name__ == "__main__":
    sys.exit(main())
