#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Build the CTPBuddy CTP ABI shim DLLs with MSVC (Windows-only).

Usage:
    python build_msvc.py [--demo]

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
SDK = os.path.join(REPO, "ctpsdk", "6.7.13_20260225")
GENERATED = os.path.join(HERE, "src", "generated")

VCVARS_CANDIDATES = [
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
]


def find_vcvars():
    env = os.environ.get("VCVARS64")
    if env and os.path.exists(env):
        return env
    for cand in VCVARS_CANDIDATES:
        if os.path.exists(cand):
            return cand
    raise SystemExit("vcvars64.bat not found; set VCVARS64 to its path")


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
    ap = argparse.ArgumentParser()
    ap.add_argument("--demo", action="store_true", help="also build shim/demo/demo_td.exe")
    args = ap.parse_args()

    # keep the generated sources in sync with the SDK headers
    subprocess.run([sys.executable, os.path.join(HERE, "codegen", "gen_shim.py")], check=True)

    vcvars = find_vcvars()
    sdk_td = os.path.join(SDK, "td", "win64")
    sdk_md = os.path.join(SDK, "md", "win64")
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
