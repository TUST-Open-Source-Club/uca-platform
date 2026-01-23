#!/usr/bin/env bash
set -euo pipefail

generate_key() {
  # 32 bytes base64
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 32
    return
  fi
  if command -v python3 >/dev/null 2>&1; then
    python3 - <<'PY'
import os, base64
print(base64.b64encode(os.urandom(32)).decode())
PY
    return
  fi
  echo "请安装 openssl 或 python3" >&2
  exit 1
}

echo "AUTH_SECRET_KEY=$(generate_key)"
echo "TLS_KEY_ENC_KEY=$(generate_key)"
