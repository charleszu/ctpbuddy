#!/bin/sh
set -eu

pass() { printf '[PASS] %s\n' "$1"; }
skip() { printf '[SKIP] %s\n' "$1"; }
fail() { printf '[FAIL] %s\n' "$1" >&2; exit 1; }

command -v ctpbuddy >/dev/null 2>&1 || fail 'Python wheel/CLI 未安装'
ctpbuddy --version >/dev/null || fail 'ctpbuddy --version'
ctpbuddy refdata show /app/refdata >/dev/null || fail 'refdata 校验'
command -v ctpbuddy-server >/dev/null 2>&1 || fail 'Rust core binary 未安装'
ctpbuddy-server --version >/dev/null 2>&1 || fail 'Rust core binary version'
python - <<'PY'
import json
import subprocess
import tempfile
import time
import threading
from urllib.request import urlopen
from ctpbuddy.sdk import Admin
from ctpbuddy.web import make_server
from ctpbuddy.journal_contract import validate_event_contract
from importlib.resources import files

vectors = json.loads(files('ctpbuddy.contracts').joinpath('journal_vectors.json').read_text(encoding='utf-8'))
for case in vectors['validation']:
    problems = validate_event_contract(case['event'])
    if (not problems) is not case['event_valid']:
        raise SystemExit('installed journal contract vector failed: ' + case['name'])

with tempfile.TemporaryDirectory(prefix='ctpbuddy-docker-') as data:
    proc = subprocess.Popen([
        'ctpbuddy-server', '--td', '127.0.0.1:0', '--admin', '127.0.0.1:5561',
        '--data-dir', data, '--refdata', '/app/refdata',
    ], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    try:
        deadline = time.time() + 10
        while time.time() < deadline:
            try:
                with Admin('127.0.0.1:5561', timeout=0.5) as admin:
                    if admin.ping().get('ok'):
                        break
            except (OSError, ConnectionError):
                time.sleep(0.1)
        else:
            raise SystemExit('core admin endpoint 未启动')
        web = make_server('127.0.0.1', 8080, '127.0.0.1:5561')
        thread = threading.Thread(target=web.serve_forever, daemon=True)
        thread.start()
        for path in ('/', '/settings.js', '/settings.css', '/live.js', '/api/live'):
            with urlopen('http://127.0.0.1:8080' + path, timeout=3) as response:
                if response.status != 200:
                    raise SystemExit('Web 状态异常: ' + path)
        web.shutdown()
        thread.join(timeout=3)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
PY
pass 'Python wheel/CLI'
pass 'installed canonical journal contract/vectors'
pass 'Rust core build/run'
pass 'refdata'
pass 'Web HTTP/assets/live API'

if command -v hugo >/dev/null 2>&1; then
  (cd /app/docs-site-oink && hugo --cleanDestinationDir --gc --minify --environment production --printPathWarnings --panicOnWarning >/dev/null) || fail 'OINK/Hugo 构建'
  pass 'OINK/Hugo'
else
  skip 'OINK/Hugo：镜像未提供 hugo 二进制；请在独立 Hugo 环境运行 docs-site-oink 构建'
fi

# This image is Linux-only and intentionally contains no Windows shim execution path.
if find /app -maxdepth 3 \( -iname '*.dll' -o -iname '*.exe' \) -print -quit | grep -q .; then
  fail 'Linux 验收镜像不应包含 Windows Shim'
fi
pass 'Windows Shim 未运行（Linux 容器边界）'
