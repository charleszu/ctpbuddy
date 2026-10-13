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

ZeroMQ is compiled from a pinned source into a STATIC library with the same
toolchain and CURVE disabled (ADR-0002) and linked into each .so: no runtime
libzmq dependency, vendor single-file shape. Requires cmake + make; set
CTPBUDDY_LIBZMQ_DIR to reuse an existing install prefix (include/ + lib/).

Outputs (under shim/bin):
    thosttraderapi_se.so   thostmduserapi_se.so    (same names as the vendor)
    demo_td                                        (with --demo)

The vendor Linux headers define TRADER_API_EXPORT as empty, so every symbol
has default visibility and the Itanium-mangled factory names are identical to
the vendor .so: a pure file replacement, rpath/LD_LIBRARY_PATH unchanged.
The generated sources are checked in; --regen re-runs codegen/gen_shim.py.
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

ZMQ_VERSION = "4.3.5"
ZMQ_SHA256 = "6653ef5910f17954861fe72332e68b03ca6e4d9c7160eb3a8de5a5a913bfab43"
ZMQ_URL = "https://github.com/zeromq/libzmq/releases/download/v%s/zeromq-%s.tar.gz" % (
    ZMQ_VERSION, ZMQ_VERSION)


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
        raise SystemExit("build step failed (%d)" % proc.returncode)
    if proc.stderr.strip():
        print(proc.stderr.strip()[-2000:])


def ensure_static_libzmq(cxx):
    """-> (include_dir, static_lib) of a CURVE-free static libzmq built from
    the pinned source with the same compiler family (ADR-0002)."""
    configured = os.environ.get("CTPBUDDY_LIBZMQ_DIR")
    if configured and os.path.exists(os.path.join(configured, "include", "zmq.h")):
        libs = sorted(glob.glob(os.path.join(configured, "lib", "*.a")))
        if libs:
            return os.path.join(configured, "include"), libs[0]
    prefix = os.path.join(THIRD_PARTY, "libzmq-%s-static" % ZMQ_VERSION)
    if not os.path.exists(os.path.join(prefix, "include", "zmq.h")):
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
                raise SystemExit("libzmq source download failed (set CTPBUDDY_LIBZMQ_DIR)")
            digest = hashlib.sha256(data).hexdigest()
            if digest != ZMQ_SHA256:
                raise SystemExit("libzmq tarball sha256 %s != pinned %s" % (digest, ZMQ_SHA256))
            with open(tarball, "wb") as f:
                f.write(data)
        if not os.path.exists(os.path.join(src_dir, "CMakeLists.txt")):
            with tarfile.open(tarball, "r:gz") as tar:
                tar.extractall(src_root)
        build_dir = os.path.join(THIRD_PARTY, "build-libzmq-linux")
        cc = shutil.which("gcc") or cxx.replace("++", "")
        cxx_path = shutil.which(cxx)
        run(["cmake", "-S", src_dir, "-B", build_dir,
             "-DCMAKE_BUILD_TYPE=Release",
             "-DCMAKE_C_COMPILER=%s" % (cc or "cc"),
             "-DCMAKE_CXX_COMPILER=%s" % (cxx_path or "c++"),
             "-DCMAKE_C_FLAGS=-fPIC", "-DCMAKE_CXX_FLAGS=-fPIC",
             "-DBUILD_SHARED=OFF", "-DBUILD_STATIC=ON",
             "-DZMQ_CURVE=OFF", "-DENABLE_CURVE=OFF", "-DWITH_LIBSODIUM=OFF",
             "-DWITH_DOCS=OFF", "-DWITH_TESTS=OFF", "-DBUILD_TESTS=OFF",
             "-DWITH_PERF_TOOL=OFF", "-DENABLE_DRAFTS=OFF",
             "-DCMAKE_INSTALL_PREFIX=%s" % prefix])
        run(["cmake", "--build", build_dir, "--config", "Release", "--target", "install"])
    libs = sorted(glob.glob(os.path.join(prefix, "lib", "*.a")))
    if not libs:
        raise SystemExit("libzmq static library missing under %s/lib" % prefix)
    print("[libzmq] static %s -> %s" % (ZMQ_VERSION, libs[0]))
    return os.path.join(prefix, "include"), libs[0]


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

    zmq_inc, zmq_lib = ensure_static_libzmq(args.cxx)
    inc = ["-I" + p for p in (HERE, os.path.join(HERE, "src"), GENERATED, sdk_td, sdk_md, zmq_inc)]
    common = [args.cxx, "-std=c++17", "-O2", "-Wall", "-Wno-unused-function", "-fPIC",
              "-fvisibility=default", "-pthread", "-DZMQ_STATIC"] + inc
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
        run(common + ["-shared", "-Wl,--no-undefined", "-Wl,-soname," + so + ".so", "-o", out]
            + objs + [zmq_lib, "-lpthread"])
        print("[built] %s" % out)

    if args.demo:
        out = os.path.join(bin_dir, "demo_td")
        run(common + [os.path.join(HERE, "demo", "demo_td.cpp"), "-o", out,
                      "-L" + bin_dir, "-l:thosttraderapi_se.so", "-l:thostmduserapi_se.so",
                      "-Wl,-rpath,$ORIGIN"])
        print("[built] %s" % out)


if __name__ == "__main__":
    main()
