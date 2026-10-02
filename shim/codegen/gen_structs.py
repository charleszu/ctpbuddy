#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""CTPBuddy codegen: CTP 6.7.13 headers -> Rust / Python / C++ struct mirrors.

Usage:
    python gen_structs.py [--sdk DIR] [--repo-root DIR]

Reads (user-supplied SDK, never committed):
    <sdk>/td/win64/ThostFtdcUserApiDataType.h
    <sdk>/td/win64/ThostFtdcUserApiStruct.h

Writes:
    core/ctpbuddy-wire/src/generated/{mod.rs, structs.rs}   Rust repr(C) mirrors
    py/ctpbuddy/generated/{__init__.py, structs.py}         Python pack/unpack
    shim/generated/registry.hpp                             C++ static_assert + ids
    registry.json                                           canonical id table

Design notes:
- CTP structs use natural alignment (no pragma pack); Rust repr(C) and Python
  native '@' struct format both match the C layout on the build platform.
- Struct ids are assigned once (0x2000 + sorted index) and never reused.
- Message ids are hand-assigned in core/ctpbuddy-wire/src/msgs.rs; this script
  only validates that every referenced struct exists.
- Structs containing nested CThostFtdc members are skipped (listed in output);
  CTP API structs are expected to be flat.
"""

import argparse
import json
import os
import re
import sys

BASE_TYPES = {
    "char": ("u8", "B", 1),
    "double": ("f64", "d", 8),
    "int": ("i32", "i", 4),
    "short": ("i16", "h", 2),
    "unsigned char": ("u8", "B", 1),
    "unsigned int": ("u32", "I", 4),
    "unsigned short": ("u16", "H", 2),
    "long long": ("i64", "q", 8),
}

# Messages referenced by the M1 wire protocol (must stay in sync with msgs.rs).
MESSAGES = [
    (0x0101, "AUTH", None),
    (0x0102, "AUTH_RSP", None),
    (0x0103, "LOGOUT", None),
    (0x0104, "LOGOUT_RSP", None),
    (0x0201, "ADMIN_REQ", None),
    (0x0202, "ADMIN_RSP", None),
    (0x1001, "REQ_USER_LOGIN", "CThostFtdcReqUserLoginField"),
    (0x1002, "RSP_USER_LOGIN", "CThostFtdcRspUserLoginField"),
    (0x1003, "RSP_USER_LOGOUT", "CThostFtdcRspInfoField"),
    (0x1004, "REQ_SETTLE_CONFIRM", "CThostFtdcSettlementInfoConfirmField"),
    (0x1005, "RSP_SETTLE_CONFIRM", "CThostFtdcRspInfoField"),
    (0x1010, "REQ_ORDER_INSERT", "CThostFtdcInputOrderField"),
    (0x1011, "RSP_ORDER_INSERT", "CThostFtdcRspInfoField"),
    (0x1012, "ERR_RTN_ORDER_INSERT", "CThostFtdcRspInfoField"),
    (0x1013, "RTN_ORDER", "CThostFtdcOrderField"),
    (0x1014, "RTN_TRADE", "CThostFtdcTradeField"),
    (0x1015, "REQ_ORDER_ACTION", "CThostFtdcInputOrderActionField"),
    (0x1016, "RSP_ORDER_ACTION", "CThostFtdcRspInfoField"),
    (0x1017, "ERR_RTN_ORDER_ACTION", "CThostFtdcRspInfoField"),
    (0x1020, "SUB_MD", None),
    (0x1021, "RSP_SUB_MD", "CThostFtdcRspInfoField"),
    (0x1022, "RTN_DEPTH_MD", "CThostFtdcDepthMarketDataField"),
    (0x1030, "RSP_ERROR", "CThostFtdcRspInfoField"),
]

TYPEDEF_BASE_RE = re.compile(
    r"typedef\s+(char|double|int|short|unsigned\s+char|unsigned\s+int|unsigned\s+short|long\s+long)"
    r"\s+(TThostFtdc\w+?)\s*(?:\[\s*(\d+)\s*\])?\s*;"
)
TYPEDEF_ALIAS_RE = re.compile(r"typedef\s+(TThostFtdc\w+)\s+(TThostFtdc\w+)\s*;")
STRUCT_RE = re.compile(r"struct\s+(CThostFtdc\w+)\s*\{(.*?)\};", re.S)
MEMBER_RE = re.compile(r"^\s*(TThostFtdc\w+|CThostFtdc\w+)\s+(\w+)\s*;", re.M)


def read_text(path):
    with open(path, "r", encoding="gb18030", errors="replace") as f:
        return f.read()


def parse_typedefs(text):
    """Returns {name: (base, size_or_None)} where base in BASE_TYPES."""
    table = {}
    for m in TYPEDEF_BASE_RE.finditer(text):
        base = re.sub(r"\s+", " ", m.group(1))
        table[m.group(2)] = (base, int(m.group(3)) if m.group(3) else None)
    # alias chains: typedef TThostFtdcA TThostFtdcB;
    for _ in range(8):
        changed = False
        for m in TYPEDEF_ALIAS_RE.finditer(text):
            src, dst = m.group(1), m.group(2)
            if src in table and dst not in table:
                table[dst] = table[src]
                changed = True
        if not changed:
            break
    return table


def parse_structs(text):
    out = {}
    for m in STRUCT_RE.finditer(text):
        name = m.group(1)
        body = m.group(2)
        members = []
        for mm in MEMBER_RE.finditer(body):
            members.append((mm.group(1), mm.group(2)))
        if members:
            out[name] = members
    return out


def resolve(tname, typedefs):
    """-> ('char', None) | ('char', 21) | ('double', None) | ('nested', name) | None"""
    t = typedefs.get(tname)
    if t is None:
        if tname.startswith("CThostFtdc"):
            return ("nested", tname)
        return None
    return t


def rust_type(spec):
    base, size = spec
    if base == "char":
        return "[u8; %d]" % size if size else "u8"
    return BASE_TYPES[base][0]


def kind_of(spec):
    """-> (python struct format kind, byte size)"""
    base, size = spec
    if base == "char":
        return ("s", size) if size else ("B", 1)
    return (BASE_TYPES[base][1], BASE_TYPES[base][2])


def gen_rust(structs, skipped, repo_root):
    lines = [
        "// @generated by shim/codegen/gen_structs.py from CTP 6.7.13 headers. DO NOT EDIT.",
        "#![allow(dead_code, non_camel_case_types, non_snake_case)]",
        "",
        "pub fn cstr(buf: &[u8]) -> String {",
        "    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());",
        "    String::from_utf8_lossy(&buf[..end]).into_owned()",
        "}",
        "",
        "pub fn set_cstr(buf: &mut [u8], s: &str) {",
        "    let bytes = s.as_bytes();",
        "    let n = bytes.len().min(buf.len());",
        "    buf[..n].copy_from_slice(&bytes[..n]);",
        "    for b in buf.iter_mut().skip(n) {",
        "        *b = 0;",
        "    }",
        "}",
        "",
    ]
    for name in sorted(structs):
        members = structs[name]
        lines.append("#[repr(C)]")
        lines.append("#[derive(Clone, Copy, Debug)]")
        lines.append("pub struct %s {" % name)
        for tname, fname in members:
            spec = resolve(tname, TYPEDEFS)
            if spec is None:
                raise SystemExit("unresolved field type %s in %s" % (tname, name))
            lines.append("    pub %s: %s," % (fname, rust_type(spec)))
        lines.append("}")
        lines.append("")
        lines.append("impl %s {" % name)
        lines.append("    pub fn zeroed() -> Self {")
        lines.append("        // SAFETY: all fields are plain integers/byte arrays.")
        lines.append("        unsafe { std::mem::zeroed() }")
        lines.append("    }")
        lines.append("}")
        lines.append("")
    if skipped:
        lines.append("// skipped (nested or unresolved members), see registry.json:")
        for name in skipped:
            lines.append("// %s" % name)
        lines.append("")
    path = os.path.join(repo_root, "core", "ctpbuddy-wire", "src", "generated", "structs.rs")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return len(lines)


def gen_python(layouts, repo_root):
    lines = [
        '"""@generated by shim/codegen/gen_structs.py from CTP 6.7.13 headers. DO NOT EDIT.',
        "",
        "CTP struct mirrors. Native alignment ('@') plus explicit trailing padding",
        "matches the C layout (verified against ctypes at generation time).",
        '"""',
        "import struct",
        "",
        "FIELD_SPECS = {",
    ]
    for name in sorted(layouts):
        rows = layouts[name]["fields"]
        lines.append("    %r: %r," % (name, rows))
    lines.append("}")
    lines.append("")
    lines.append("_FMT = {")
    for name in sorted(layouts):
        lines.append("    %r: %r," % (name, layouts[name]["fmt"]))
    lines.append("}")
    lines.append("")
    lines.append("SIZES = {")
    for name in sorted(layouts):
        lines.append("    %r: %d," % (name, layouts[name]["size"]))
    lines.append("}")
    lines.append("_CACHE = {}")
    lines.append("")
    lines.append("")
    lines.append("def _packer(name):")
    lines.append("    s = _CACHE.get(name)")
    lines.append("    if s is None:")
    lines.append("        s = struct.Struct(_FMT[name])")
    lines.append("        _CACHE[name] = s")
    lines.append("    return s")
    lines.append("")
    lines.append("")
    lines.append("def verify_layout():")
    lines.append("    \"\"\"Every struct must pack to exactly SIZES[name]. Used by tests.\"\"\"")
    lines.append("    for name, size in SIZES.items():")
    lines.append("        got = _packer(name).size")
    lines.append("        assert got == size, '%s: %d != %d' % (name, got, size)")
    lines.append("    return len(SIZES)")
    lines.append("")
    lines.append("")
    lines.append("def pack(name, **kw):")
    lines.append("    rows = FIELD_SPECS[name]")
    lines.append("    values = []")
    lines.append("    for fname, kind, size in rows:")
    lines.append("        v = kw.get(fname)")
    lines.append("        if kind == 's':")
    lines.append("            if v is None:")
    lines.append("                v = b''")
    lines.append("            if isinstance(v, str):")
    lines.append("                v = v.encode('utf-8', 'replace')")
    lines.append("            v = v[:size].ljust(size, b'\\x00')")
    lines.append("        elif v is None:")
    lines.append("            v = 0")
    lines.append("        elif isinstance(v, str):")
    lines.append("            v = ord(v[0]) if v else 0")
    lines.append("        values.append(v)")
    lines.append("    return _packer(name).pack(*values)")
    lines.append("")
    lines.append("")
    lines.append("def unpack(name, buf):")
    lines.append("    rows = FIELD_SPECS[name]")
    lines.append("    values = _packer(name).unpack_from(buf)")
    lines.append("    out = {}")
    lines.append("    for (fname, kind, _size), v in zip(rows, values):")
    lines.append("        if kind == 's':")
    lines.append("            v = v.split(b'\\x00')[0].decode('utf-8', 'replace')")
    lines.append("        out[fname] = v")
    lines.append("    return out")
    path = os.path.join(repo_root, "py", "ctpbuddy", "generated", "structs.py")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    init = [
        '"""@generated struct mirrors for CTPBuddy wire protocol. DO NOT EDIT."""',
        "from .structs import FIELD_SPECS, SIZES, pack, unpack, verify_layout  # noqa: F401",
        "",
    ]
    with open(os.path.join(os.path.dirname(path), "__init__.py"), "w", encoding="utf-8") as f:
        f.write("\n".join(init))


def gen_cpp(layouts, skipped, repo_root):
    lines = [
        "// @generated by shim/codegen/gen_structs.py from CTP 6.7.13 headers. DO NOT EDIT.",
        "#pragma once",
        "#include <cstdint>",
        "#include \"ThostFtdcUserApiStruct.h\"",
        "",
        "namespace ctpbuddy {",
        "",
        "enum StructId : uint16_t {",
    ]
    for i, name in enumerate(sorted(layouts)):
        lines.append("    k%s = 0x%04X," % (name, 0x2000 + i))
    lines.append("};")
    lines.append("")
    lines.append("// sizeof assertions: build fails loudly on any layout drift.")
    for name in sorted(layouts):
        lines.append(
            "static_assert(sizeof(%s) == %d, \"%s size mismatch\");"
            % (name, layouts[name]["size"], name)
        )
    lines.append("")
    lines.append("}  // namespace ctpbuddy")
    lines.append("")
    path = os.path.join(repo_root, "shim", "generated", "registry.hpp")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


def build_layout(members):
    """-> (rows, fmt, size) with C-correct trailing padding.

    rows: [(field, kind, size)]; fmt: python struct format incl. trailing pad.
    Python struct '@' aligns between fields but does NOT pad the total to the
    struct alignment, so trailing padding is added explicitly. ctypes (native
    C layout) is the cross-check authority.
    """
    import ctypes
    import struct as _struct

    rows = []
    for tname, _fname in members:
        spec = resolve(tname, TYPEDEFS)
        if spec is None:
            raise SystemExit("unresolved field type %s" % tname)
        kind, size = kind_of(spec)
        rows.append((kind, size))

    body = "".join(("%ds" % s) if k == "s" else k for k, s in rows)
    max_align = max([1] + [1 if k == "s" else min(s, 8) for k, s in rows])
    raw = _struct.calcsize("@" + body)
    pad = (-raw) % max_align
    fmt = "@" + body + ("%dx" % pad if pad else "")
    size = raw + pad

    def ctype_of(kind, size):
        if kind == "s":
            return ctypes.c_char * size
        if kind == "d":
            return ctypes.c_double
        if kind == "i":
            return ctypes.c_int32
        if kind == "h":
            return ctypes.c_int16
        if kind == "B":
            return ctypes.c_uint8
        if kind == "I":
            return ctypes.c_uint32
        if kind == "H":
            return ctypes.c_uint16
        if kind == "q":
            return ctypes.c_int64
        raise SystemExit("unknown struct kind %r" % kind)

    fields = [("f%d" % i, ctype_of(k, s)) for i, (k, s) in enumerate(rows)]
    cls = type("T", (ctypes.Structure,), {"_fields_": fields})
    if ctypes.sizeof(cls) != size:
        raise SystemExit("layout disagreement: ctypes=%d computed=%d" % (ctypes.sizeof(cls), size))
    return rows, fmt, size


def gen_registry_json(layouts, skipped, repo_root):
    out = {"wire_version": 1, "structs": {}, "messages": {}}
    for i, name in enumerate(sorted(layouts)):
        fields = [
            {"name": fname, "kind": kind, "size": size}
            for fname, kind, size in layouts[name]["fields"]
        ]
        out["structs"][name] = {
            "id": 0x2000 + i,
            "size": layouts[name]["size"],
            "fields": fields,
        }
    for mid, mname, mstruct in MESSAGES:
        out["messages"][mname] = {
            "id": mid,
            "struct": mstruct,
            "payload": "json" if mstruct is None else "raw-struct",
        }
    out["skipped"] = skipped
    path = os.path.join(repo_root, "registry.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=2, ensure_ascii=False)


def main():
    global TYPEDEFS
    ap = argparse.ArgumentParser()
    ap.add_argument("--sdk", default=os.environ.get("CTPBUDDY_SDK"))
    ap.add_argument("--repo-root", default=os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
    args = ap.parse_args()
    if not args.sdk:
        # default: newest version dir under ctpsdk/
        base = os.path.join(args.repo_root, "ctpsdk")
        vers = sorted(d for d in os.listdir(base) if os.path.isdir(os.path.join(base, d)))
        if not vers:
            raise SystemExit("no SDK found under ctpsdk/; use --sdk or set CTPBUDDY_SDK")
        args.sdk = os.path.join(base, vers[-1])
    hdr = os.path.join(args.sdk, "td", "win64")
    if not os.path.isdir(hdr):
        raise SystemExit("header dir not found: %s" % hdr)

    typedefs = parse_typedefs(read_text(os.path.join(hdr, "ThostFtdcUserApiDataType.h")))
    structs = parse_structs(read_text(os.path.join(hdr, "ThostFtdcUserApiStruct.h")))
    TYPEDEFS = typedefs

    skipped = []
    for name in sorted(structs):
        for tname, _f in structs[name]:
            if tname.startswith("CThostFtdc"):
                skipped.append(name + " (nested %s)" % tname)
                break
            if resolve(tname, typedefs) is None:
                skipped.append(name + " (unresolved %s)" % tname)
                break
    for s in list(structs):
        if any(s in x for x in skipped):
            structs.pop(s)

    # validate message table
    for _mid, mname, mstruct in MESSAGES:
        if mstruct and mstruct not in structs:
            raise SystemExit("message %s references missing struct %s" % (mname, mstruct))

    layouts = {}
    for name in sorted(structs):
        rows, fmt, size = build_layout(structs[name])
        named_rows = [(fname, kind, sz) for (_t, fname), (kind, sz) in zip(structs[name], rows)]
        layouts[name] = {"fields": named_rows, "fmt": fmt, "size": size}

    n = gen_rust(structs, skipped, args.repo_root)
    gen_python(layouts, args.repo_root)
    gen_cpp(layouts, skipped, args.repo_root)
    gen_registry_json(layouts, skipped, args.repo_root)

    print("sdk:        %s" % args.sdk)
    print("typedefs:   %d" % len(typedefs))
    print("structs:    %d generated, %d skipped" % (len(structs), len(skipped)))
    if skipped:
        print("skipped:")
        for s in skipped:
            print("  - %s" % s)
    print("rust lines: %d" % n)
    print("ok")


if __name__ == "__main__":
    sys.exit(main())
