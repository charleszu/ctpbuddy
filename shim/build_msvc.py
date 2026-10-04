#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Build the CTPBuddy CTP ABI shim DLLs with MSVC (Windows-only).

Usage:
    python build_msvc.py [--demo] [--sdk DIR]

SDK headers (CTP 6.7.13): --sdk / CTPBUDDY_SDK, default ctpsdk/6.7.13_20260225
(gitignored, user-supplied). Both layouts are accepted:
    <sdk>/td/win64/*.h + <sdk>/md/win64/*.h      the vendor zip layout
    <sdk>/*.h                                      a flat directory, e.g. the
                                                   checked-in copies under
                                                   docs/api-doc-html/files/
                                                   (same bytes, CRLF) -- what CI
                                                   uses, since ctpsdk/ is not
                                                   committed.
Only the headers are needed: the shim never links the vendor .lib.

Outputs (all under shim/):
    bin/thosttraderapi_se.dll + lib/thosttraderapi_se.lib
    bin/thostmduserapi_se.dll  + lib/thostmduserapi_se.lib
    bin/demo_td.exe            (with --demo)

The DLLs are byte-layout drop-in replacements for the CTP 6.7.13 vendor DLLs
(same names, same mangled factory symbols, same Struct sizes -- enforced at
compile time by shim/generated/registry.hpp's static_asserts).
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

# VS 2022 installs under "Program Files" (64-bit, e.g. GitHub's windows-latest
# Enterprise image) or "Program Files (x86)" (Build Tools); probe both.
VCVARS_CANDIDATES = [
    r"%s\Microsoft Visual Studio\2022\%s\VC\Auxiliary\Build\vcvars64.bat" % (pf, ed)
    for ed in ("BuildTools", "Enterprise", "Professional", "Community")
    for pf in (r"C:\Program Files (x86)", r"C:\Program Files")
]
VSWHERE = r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"


def find_vcvars():
    env = os.environ.get("VCVARS64")
    if env and os.path.exists(env):
        return env
    for cand in VCVARS_CANDIDATES:
        if os.path.exists(cand):
            return cand
    if os.path.exists(VSWHERE):
        proc = subprocess.run([VSWHERE, "-latest", "-products", "*", "-requires",
                               "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                               "-property", "installationPath"],
                              capture_output=True, text=True, errors="replace")
        root = proc.stdout.strip().splitlines()
        if proc.returncode == 0 and root:
            cand = os.path.join(root[0], "VC", "Auxiliary", "Build", "vcvars64.bat")
            if os.path.exists(cand):
                return cand
    raise SystemExit("vcvars64.bat not found; set VCVARS64 to its path")


def sdk_include_dirs(sdk):
    """-> (td_dir, md_dir) holding the vendor headers; fails loudly if absent."""
    td = os.path.join(sdk, "td", "win64")
    md = os.path.join(sdk, "md", "win64")
    if not os.path.exists(os.path.join(td, "ThostFtdcTraderApi.h")):
        td = md = sdk  # flat layout
    for d, h in ((td, "ThostFtdcTraderApi.h"), (md, "ThostFtdcMdApi.h"), (td, "ThostFtdcUserApiStruct.h")):
        if not os.path.exists(os.path.join(d, h)):
            raise SystemExit("CTP header %s not found under %s (use --sdk / CTPBUDDY_SDK)" % (h, sdk))
    return td, md


def run_cl(vcvars, args):
    cmd = 'call "%s" >nul && cl %s' % (vcvars, args)
    proc = subprocess.run(cmd, shell=True, capture_output=True, text=True, errors="replace")
    if proc.returncode != 0:
        print(proc.stdout)
        print(proc.stderr, file=sys.stderr)
        raise SystemExit("cl failed (%d)" % proc.returncode)
    # MSVC prints each source name to stdout; keep the tail visible.
    print(proc.stdout.strip()[-2000:])


def main():
    # cl's (GBK/locale) output is echoed through our stdout, which may be a
    # different code page (CI console, redirected pipe): never die on that.
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(errors="replace")
    ap = argparse.ArgumentParser()
    ap.add_argument("--demo", action="store_true", help="also build shim/demo/demo_td.exe")
    ap.add_argument("--sdk", default=os.environ.get("CTPBUDDY_SDK") or DEFAULT_SDK,
                    help="CTP 6.7.13 header dir (vendor layout or flat); env CTPBUDDY_SDK")
    args = ap.parse_args()
    sdk = os.path.abspath(args.sdk)
    sdk_td, sdk_md = sdk_include_dirs(sdk)

    # keep the generated sources in sync with the SDK headers
    subprocess.run([sys.executable, os.path.join(HERE, "codegen", "gen_shim.py"), "--sdk", sdk], check=True)

    vcvars = find_vcvars()
    # shim root is on the path so "generated/registry.hpp" resolves to
    # shim/generated/registry.hpp while "api_td.hpp" resolves from the
    # including file's own directory (shim/src/generated).
    inc = '/I"%s" /I"%s" /I"%s" /I"%s" /I"%s"' % (
        HERE, os.path.join(HERE, "src"), GENERATED, sdk_td, sdk_md)
    common = "/nologo /LD /EHsc /std:c++17 /O2 /W3 /MD /utf-8 %s" % inc
    bin_dir = os.path.join(HERE, "bin")
    lib_dir = os.path.join(HERE, "lib")
    os.makedirs(bin_dir, exist_ok=True)
    os.makedirs(lib_dir, exist_ok=True)

    targets = [
        ("td", ["api_core.cpp", "generated/api_td_reqs.cpp", "generated/dispatch_td.cpp",
                "exports_td.cpp"], "ISLIB /DWIN32 /DLIB_TRADER_API_EXPORT", "thosttraderapi_se"),
        ("md", ["api_core.cpp", "generated/api_md_reqs.cpp", "generated/dispatch_md.cpp",
                "exports_md.cpp", "api_md.cpp"], "ISLIB /DWIN32 /DLIB_MD_API_EXPORT", "thostmduserapi_se"),
    ]
    for name, srcs, export_macro, dll in targets:
        obj_dir = os.path.join(HERE, "build", "obj_" + name)
        os.makedirs(obj_dir, exist_ok=True)
        objs = []
        for src in srcs:
            src_path = os.path.join(HERE, "src", src)
            stem = os.path.splitext(os.path.basename(src))[0]
            obj = os.path.join(obj_dir, stem + ".obj")
            objs.append(obj)
            cl = '%s /D%s /Fo"%s" /c "%s"' % (common, export_macro, obj, src_path)
            run_cl(vcvars, cl)
        out_dll = os.path.join(bin_dir, dll + ".dll")
        out_lib = os.path.join(lib_dir, dll + ".lib")
        cl = '%s /D%s /Fe"%s" %s /link /OUT:"%s" /IMPLIB:"%s"' % (
            common, export_macro, out_dll, " ".join('"%s"' % o for o in objs), out_dll, out_lib)
        run_cl(vcvars, cl)
        if not os.path.exists(out_lib):
            raise SystemExit("import lib missing: %s" % out_lib)
        print("[built] %s (+ %s)" % (out_dll, out_lib))

    if args.demo:
        obj_dir = os.path.join(HERE, "build", "obj_demo")
        os.makedirs(obj_dir, exist_ok=True)
        demo = os.path.join(HERE, "demo", "demo_td.cpp")
        out = os.path.join(bin_dir, "demo_td.exe")
        cl = '/nologo /EHsc /std:c++17 /O2 /W3 /MD /utf-8 %s /Fo"%s" /Fe"%s" "%s" /link /OUT:"%s" "%s" "%s"' % (
            inc, obj_dir, out, demo, out,
            os.path.join(lib_dir, "thosttraderapi_se.lib"),
            os.path.join(lib_dir, "thostmduserapi_se.lib"))
        run_cl(vcvars, cl)
        print("[built] %s" % out)


if __name__ == "__main__":
    main()
