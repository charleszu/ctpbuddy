#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Build the CTPBuddy CTP ABI shim shared objects with g++ (Linux / WSL).

Usage:
    python3 build_linux.py [--demo] [--sdk DIR] [--regen] [--cxx g++]

Mirror of build_msvc.py. SDK headers (CTP 6.7.13): --sdk / CTPBUDDY_SDK,
default ctpsdk/6.7.13_20260225. Accepted layouts:
    <sdk>/td/linux64/*.h + <sdk>/md/linux64/*.h   the vendor zip layout
    <sdk>/td/win64/*.h  + <sdk>/md/win64/*.h      (same API surface)
    <sdk>/*.h                                      flat, e.g. docs/api-doc-html/files
Only headers are needed: the shim never links the vendor .so.

Outputs (under shim/bin):
    thosttraderapi_se.so   thostmduserapi_se.so    (same names as the vendor)
    demo_td                                        (with --demo)

The vendor Linux headers define TRADER_API_EXPORT as empty, so every symbol
has default visibility and the Itanium-mangled factory names are identical to
the vendor .so: a pure file replacement, rpath/LD_LIBRARY_PATH unchanged.
The generated sources are checked in; --regen re-runs codegen/gen_shim.py.
"""
import argparse
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
DEFAULT_SDK = os.path.join(REPO, "ctpsdk", "6.7.13_20260225")
GENERATED = os.path.join(HERE, "src", "generated")


def sdk_include_dirs(sdk):
    """-> (td_dir, md_dir) holding the vendor headers; fails loudly if absent."""
    for plat in ("linux64", "win64"):
        td = os.path.join(sdk, "td", plat)
        md = os.path.join(sdk, "md", plat)
        if os.path.exists(os.path.join(td, "ThostFtdcTraderApi.h")):
            break
    else:
        td = md = sdk  # flat layout
    for d, h in ((td, "ThostFtdcTraderApi.h"), (md, "ThostFtdcMdApi.h"), (td, "ThostFtdcUserApiStruct.h")):
        if not os.path.exists(os.path.join(d, h)):
            raise SystemExit("CTP header %s not found under %s (use --sdk / CTPBUDDY_SDK)" % (h, sdk))
    return td, md


def run(cmd):
    proc = subprocess.run(cmd, capture_output=True, text=True, errors="replace")
    if proc.returncode != 0:
        print(" ".join(cmd))
        print(proc.stdout)
        print(proc.stderr, file=sys.stderr)
        raise SystemExit("compiler failed (%d)" % proc.returncode)
    if proc.stderr.strip():
        print(proc.stderr.strip()[-2000:])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--demo", action="store_true", help="also build shim/bin/demo_td")
    ap.add_argument("--sdk", default=os.environ.get("CTPBUDDY_SDK") or DEFAULT_SDK)
    ap.add_argument("--regen", action="store_true", help="re-run codegen/gen_shim.py first")
    ap.add_argument("--cxx", default=os.environ.get("CXX") or "g++")
    args = ap.parse_args()
    if not shutil.which(args.cxx):
        raise SystemExit("C++ compiler %r not found (apt install g++)" % args.cxx)
    sdk = os.path.abspath(args.sdk)
    sdk_td, sdk_md = sdk_include_dirs(sdk)

    if args.regen:
        subprocess.run([sys.executable, os.path.join(HERE, "codegen", "gen_shim.py"), "--sdk", sdk], check=True)

    inc = ["-I" + p for p in (HERE, os.path.join(HERE, "src"), GENERATED, sdk_td, sdk_md)]
    common = [args.cxx, "-std=c++17", "-O2", "-Wall", "-Wno-unused-function", "-fPIC",
              "-fvisibility=default", "-pthread"] + inc
    bin_dir = os.path.join(HERE, "bin")
    os.makedirs(bin_dir, exist_ok=True)

    targets = [
        ("td", ["api_core.cpp", "generated/api_td_reqs.cpp", "generated/dispatch_td.cpp", "exports_td.cpp"],
         "thosttraderapi_se"),
        ("md", ["api_core.cpp", "generated/api_md_reqs.cpp", "generated/dispatch_md.cpp", "exports_md.cpp",
                "api_md.cpp"], "thostmduserapi_se"),
    ]
    for name, srcs, so in targets:
        obj_dir = os.path.join(HERE, "build", "obj_linux_" + name)
        os.makedirs(obj_dir, exist_ok=True)
        objs = []
        for src in srcs:
            obj = os.path.join(obj_dir, os.path.splitext(os.path.basename(src))[0] + ".o")
            objs.append(obj)
            run(common + ["-c", os.path.join(HERE, "src", src), "-o", obj])
        out = os.path.join(bin_dir, so + ".so")
        run(common + ["-shared", "-Wl,--no-undefined", "-Wl,-soname," + so + ".so", "-o", out] + objs)
        print("[built] %s" % out)

    if args.demo:
        out = os.path.join(bin_dir, "demo_td")
        run(common + [os.path.join(HERE, "demo", "demo_td.cpp"), "-o", out,
                      "-L" + bin_dir, "-l:thosttraderapi_se.so", "-l:thostmduserapi_se.so",
                      "-Wl,-rpath,$ORIGIN"])
        print("[built] %s" % out)


if __name__ == "__main__":
    main()
