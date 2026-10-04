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
- Message ids are hand-assigned in gen_shim.py's MSG table (which syncs them
  into core/ctpbuddy-wire/src/msgs.rs / api_core.hpp / wire.py); this script
  imports that table and only adds the payload-struct column, validating that
  every referenced struct exists.
- Structs containing nested CThostFtdc members are skipped (listed in output);
  CTP API structs are expected to be flat.
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_shim import MSG  # noqa: E402  single source of wire message ids

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

# Payload struct per wire message, as the shim dispatch tables consume it
# (gen_shim.TD_DISPATCH / MD_DISPATCH). None = JSON / empty / packed-array /
# composite payload. Ids come from gen_shim.MSG -- never list a number here.
# Every name must exist in MSG; every MSG entry gets a registry.json row
# (struct None when not listed).
MESSAGE_STRUCTS = {
    "REQ_USER_LOGIN": "CThostFtdcReqUserLoginField",
    "RSP_USER_LOGIN": "CThostFtdcRspUserLoginField",
    "REQ_USER_LOGOUT": "CThostFtdcUserLogoutField",
    "RSP_USER_LOGOUT": "CThostFtdcUserLogoutField",
    "REQ_SETTLE_CONFIRM": "CThostFtdcSettlementInfoConfirmField",
    "RSP_SETTLE_CONFIRM": "CThostFtdcSettlementInfoConfirmField",
    "REQ_ORDER_INSERT": "CThostFtdcInputOrderField",
    # RSP_ORDER_INSERT / RSP_ORDER_ACTION: empty payload (shim echoes the
    # cached input); ERR_RTN_*: input struct ++ RspInfoField (composite).
    "RTN_ORDER": "CThostFtdcOrderField",
    "RTN_TRADE": "CThostFtdcTradeField",
    "REQ_ORDER_ACTION": "CThostFtdcInputOrderActionField",
    # SUB_MD / UNSUB_MD: packed array of SpecificInstrumentField; one RSP per
    # element.
    "RSP_SUB_MD": "CThostFtdcSpecificInstrumentField",
    "RSP_UNSUB_MD": "CThostFtdcSpecificInstrumentField",
    "RTN_DEPTH_MD": "CThostFtdcDepthMarketDataField",
    "RSP_ERROR": "CThostFtdcRspInfoField",
    "REQ_QRY_SETTLEMENT_INFO": "CThostFtdcQrySettlementInfoField",
    "RSP_QRY_SETTLEMENT_INFO": "CThostFtdcSettlementInfoField",
    "REQ_QRY_INSTRUMENT": "CThostFtdcQryInstrumentField",
    "RSP_QRY_INSTRUMENT": "CThostFtdcInstrumentField",
    "REQ_QRY_TRADING_ACCOUNT": "CThostFtdcQryTradingAccountField",
    "RSP_QRY_TRADING_ACCOUNT": "CThostFtdcTradingAccountField",
    "REQ_QRY_INVESTOR_POSITION": "CThostFtdcQryInvestorPositionField",
    "RSP_QRY_INVESTOR_POSITION": "CThostFtdcInvestorPositionField",
    "REQ_QRY_ORDER": "CThostFtdcQryOrderField",
    "RSP_QRY_ORDER": "CThostFtdcOrderField",
    "REQ_QRY_TRADE": "CThostFtdcQryTradeField",
    "RSP_QRY_TRADE": "CThostFtdcTradeField",
    "REQ_QRY_INSTRUMENT_MARGIN_RATE": "CThostFtdcQryInstrumentMarginRateField",
    "RSP_QRY_INSTRUMENT_MARGIN_RATE": "CThostFtdcInstrumentMarginRateField",
    "REQ_QRY_INSTRUMENT_COMMISSION_RATE": "CThostFtdcQryInstrumentCommissionRateField",
    "RSP_QRY_INSTRUMENT_COMMISSION_RATE": "CThostFtdcInstrumentCommissionRateField",
    "REQ_QRY_INSTRUMENT_ORDER_COMM_RATE": "CThostFtdcQryInstrumentOrderCommRateField",
    "RSP_QRY_INSTRUMENT_ORDER_COMM_RATE": "CThostFtdcInstrumentOrderCommRateField",
    "REQ_QRY_BROKER_TRADING_PARAMS": "CThostFtdcQryBrokerTradingParamsField",
    "RSP_QRY_BROKER_TRADING_PARAMS": "CThostFtdcBrokerTradingParamsField",
    "REQ_QRY_INVESTOR_POSITION_DETAIL": "CThostFtdcQryInvestorPositionDetailField",
    "RSP_QRY_INVESTOR_POSITION_DETAIL": "CThostFtdcInvestorPositionDetailField",
    "REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN": "CThostFtdcQryInvestorProductGroupMarginField",
    "RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN": "CThostFtdcInvestorProductGroupMarginField",
}


def message_table():
    """-> [(id, name, struct_or_None)] sorted by id, derived from gen_shim.MSG."""
    unknown = sorted(set(MESSAGE_STRUCTS) - set(MSG))
    if unknown:
        raise SystemExit("MESSAGE_STRUCTS names not in gen_shim.MSG: %s" % ", ".join(unknown))
    return [(mid, name, MESSAGE_STRUCTS.get(name)) for name, mid in sorted(MSG.items(), key=lambda kv: kv[1])]


MESSAGES = message_table()

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
        "/// Read a CTP char field (GBK on the wire) as a Rust string.",
        "pub fn cstr(buf: &[u8]) -> String {",
        "    crate::gbk::read_field(buf)",
        "}",
        "",
        "/// Write a Rust string into a CTP char field as GBK, NUL-padded.",
        "pub fn set_cstr(buf: &mut [u8], s: &str) {",
        "    crate::gbk::write_field(buf, s)",
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
        rows, _, wire_size = build_layout(members)
        offset = 0
        fields = []
        for (_, field), (kind, size) in zip(members, rows):
            alignment = 1 if kind == 's' else min(size, 8)
            offset = (offset + alignment - 1) // alignment * alignment
            fields.append((field, kind, size, offset))
            offset += size
        lines.append("impl crate::WireStruct for %s {" % name)
        lines.append("    fn decode(buf: &[u8]) -> Option<Self> {")
        lines.append("        if buf.len() != %d { return None; }" % wire_size)
        lines.append("        Some(Self {")
        for field, kind, size, offset in fields:
            if kind == 's':
                expr = "buf[%d..%d].try_into().ok()?" % (offset, offset + size)
            elif kind == 'B':
                expr = "buf[%d]" % offset
            else:
                numeric = {'d': 'f64', 'i': 'i32', 'h': 'i16', 'I': 'u32', 'H': 'u16', 'q': 'i64'}[kind]
                expr = "%s::from_le_bytes(buf[%d..%d].try_into().ok()?)" % (numeric, offset, offset + size)
            lines.append("            %s: %s," % (field, expr))
        lines.extend(["        })", "    }", "    fn encode(&self) -> Vec<u8> {"])
        lines.append("        let mut buf = vec![0u8; %d];" % wire_size)
        for field, kind, size, offset in fields:
            if kind == 'B':
                lines.append("        buf[%d] = self.%s;" % (offset, field))
            else:
                expr = "self.%s" % field if kind == 's' else "self.%s.to_le_bytes()" % field
                lines.append("        buf[%d..%d].copy_from_slice(&%s);" % (offset, offset + size, expr))
        lines.extend(["        buf", "    }", "}"])
        lines.append("const _: () = assert!(std::mem::size_of::<%s>() == %d);" % (name, wire_size))
        for field, _, _, offset in fields:
            lines.append("const _: () = assert!(std::mem::offset_of!(%s, %s) == %d);" % (name, field, offset))
        lines.append("")
    lines.extend(["#[cfg(test)]", "#[test]", "fn all_wire_layouts_roundtrip_with_zero_padding() {"])
    for name in sorted(structs):
        rows, _, wire_size = build_layout(structs[name])
        lines.append("    {")
        lines.append("        let input = vec![0x35u8; %d];" % wire_size)
        lines.append("        let value = crate::struct_from_bytes::<%s>(&input).unwrap();" % name)
        lines.append("        let encoded = crate::struct_to_bytes(&value);")
        lines.append("        let mut expected = vec![0u8; %d];" % wire_size)
        offset = 0
        for kind, size in rows:
            alignment = 1 if kind == 's' else min(size, 8)
            offset = (offset + alignment - 1) // alignment * alignment
            lines.append("        expected[%d..%d].fill(0x35);" % (offset, offset + size))
            offset += size
        lines.append('        assert_eq!(encoded, expected, "%s");' % name)
        lines.append("        assert!(crate::struct_from_bytes::<%s>(&input[..input.len() - 1]).is_none());" % name)
        lines.append("    }")
    lines.append("}")
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
    lines.append("                v = v.encode('gbk', 'replace')")
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
    lines.append("        if kind == 's' and not (name == 'CThostFtdcSettlementInfoField' and fname == 'Content'):")
    lines.append("            v = v.split(b'\\x00')[0].decode('gbk', 'replace')")
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
