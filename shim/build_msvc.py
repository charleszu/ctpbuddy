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

ZeroMQ is compiled from a pinned source into a STATIC library with the same
MSVC toolchain (via vcvars) and CURVE disabled (ADR-0002), then linked into
each DLL: the shim keeps the vendor shape of a single self-contained
thost*_se.dll with no runtime libzmq dependency. Requires CMake on PATH
(within or outside the VS prompt). Set CTPBUDDY_LIBZMQ_DIR to reuse an
existing install prefix (include/ + lib/).

Outputs (all under shim/):
    bin/thosttraderapi_se.dll + lib/thosttraderapi_se.lib
    bin/thostmduserapi_se.dll  + lib/thostmduserapi_se.lib
    bin/demo_td.exe            (with --demo)

The DLLs are byte-layout drop-in replacements for the CTP 6.7.13 vendor DLLs
(same names, same mangled factory symbols, same Struct sizes -- enforced at
compile time by shim/generated/registry.hpp's static_asserts).
"""
import argparse
import glob
import hashlib
import os
import shutil
import subprocess
import sys
import tarfile
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
DEFAULT_SDK = os.path.join(REPO, "ctpsdk", "6.7.13_20260225")
GENERATED = os.path.join(HERE, "src", "generated")
THIRD_PARTY = os.path.join(HERE, "third_party")

# Latest upstream release at the time this pin was made; the SHA256 fixes the
# exact bytes so every build site compiles the identical source.
ZMQ_VERSION = "4.3.5"
ZMQ_SHA256 = "6653ef5910f17954861fe72332e68b03ca6e4d9c7160eb3a8de5a5a913bfab43"
ZMQ_URL = "https://github.com/zeromq/libzmq/releases/download/v%s/zeromq-%s.tar.gz" % (
    ZMQ_VERSION, ZMQ_VERSION)

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


def run_shell(vcvars, cmd, what):
    proc = subprocess.run('call "%s" >nul && %s' % (vcvars, cmd), shell=True,
                          capture_output=True, text=True, errors="replace")
    if proc.returncode != 0:
        print(proc.stdout)
        print(proc.stderr, file=sys.stderr)
        raise SystemExit("%s failed (%d)" % (what, proc.returncode))
    print(proc.stdout.strip()[-2000:])


def ensure_static_libzmq(vcvars):
    """-> install prefix (include/ + lib/) of a static, CURVE-free libzmq built
    from the pinned source with the SAME MSVC toolchain as the shim (ADR-0002:
    single-DLL drop-in shape, no runtime libzmq, no curve surface)."""
    configured = os.environ.get("CTPBUDDY_LIBZMQ_DIR")
    if configured and os.path.exists(os.path.join(configured, "include", "zmq.h")):
        return configured
    prefix = os.path.join(THIRD_PARTY, "libzmq-%s-static" % ZMQ_VERSION)
    if os.path.exists(os.path.join(prefix, "include", "zmq.h")):
        return prefix

    src_root = os.path.join(THIRD_PARTY, "src")
    os.makedirs(src_root, exist_ok=True)
    src_dir = os.path.join(src_root, "zeromq-" + ZMQ_VERSION)
    tarball = os.path.join(src_root, "zeromq-%s.tar.gz" % ZMQ_VERSION)
    if not os.path.exists(tarball):
        print("[libzmq] downloading %s" % ZMQ_URL)
        data = None
        for attempt in range(5):
            try:
                data = urllib.request.urlopen(ZMQ_URL, timeout=180).read()
                break
            except OSError as exc:
                print("[libzmq] download retry %d: %s" % (attempt, exc))
        if data is None:
            raise SystemExit("libzmq source download failed (set CTPBUDDY_LIBZMQ_DIR to a local prefix)")
        digest = hashlib.sha256(data).hexdigest()
        if digest != ZMQ_SHA256:
            raise SystemExit("libzmq tarball sha256 %s != pinned %s" % (digest, ZMQ_SHA256))
        with open(tarball, "wb") as f:
            f.write(data)
    if not os.path.exists(os.path.join(src_dir, "CMakeLists.txt")):
        with tarfile.open(tarball, "r:gz") as tar:
            tar.extractall(src_root)

    build_dir = os.path.join(THIRD_PARTY, "build-libzmq-msvc")
    run_shell(vcvars,
              'cmake -S "%s" -B "%s" -A x64 -DBUILD_SHARED=OFF -DBUILD_STATIC=ON '
              '-DZMQ_CURVE=OFF -DENABLE_CURVE=OFF -DWITH_LIBSODIUM=OFF '
              '-DWITH_DOCS=OFF -DWITH_TESTS=OFF -DBUILD_TESTS=OFF -DWITH_PERF_TOOL=OFF '
              '-DENABLE_DRAFTS=OFF -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL '
              '-DCMAKE_INSTALL_PREFIX="%s"' % (src_dir, build_dir, prefix),
              "cmake configure (libzmq)")
    run_shell(vcvars, 'cmake --build "%s" --config Release --target install' % build_dir,
              "cmake build (libzmq)")
    libs = glob.glob(os.path.join(prefix, "lib", "*.lib"))
    if not libs or glob.glob(os.path.join(prefix, "**", "*.dll"), recursive=True):
        raise SystemExit("libzmq install did not produce a static-only prefix: %s" % prefix)
    platform = os.path.join(build_dir, "platform.hpp")
    if os.path.exists(platform):
        text = open(platform, encoding="utf-8", errors="replace").read()
        if "ZMQ_HAVE_CURVE 1" in text:
            raise SystemExit("libzmq platform.hpp reports CURVE enabled despite ZMQ_CURVE=OFF")
    print("[libzmq] static %s -> %s" % (ZMQ_VERSION, libs[0]))
    return prefix


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
    zmq_prefix = ensure_static_libzmq(vcvars)
    zmq_libs = glob.glob(os.path.join(zmq_prefix, "lib", "*.lib"))
    # shim root is on the path so "generated/registry.hpp" resolves to
    # shim/generated/registry.hpp while "api_td.hpp" resolves from the
    # including file's own directory (shim/src/generated).
    inc = '/I"%s" /I"%s" /I"%s" /I"%s" /I"%s" /I"%s" /DZMQ_STATIC' % (
        HERE, os.path.join(HERE, "src"), GENERATED, sdk_td, sdk_md,
        os.path.join(zmq_prefix, "include"))
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
        cl = ('%s /D%s /Fe"%s" %s "%s" ws2_32.lib iphlpapi.lib advapi32.lib '
              '/link /OUT:"%s" /IMPLIB:"%s"') % (
            common, export_macro, out_dll, " ".join('"%s"' % o for o in objs),
            zmq_libs[0], out_dll, out_lib)
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
