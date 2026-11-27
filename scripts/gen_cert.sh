#!/usr/bin/env bash
# Generate self-signed TLS cert — CN=update.microsoft.com (T1036.005)
# Output: cert.pem + key.pem at repo root (both in .gitignore)
set -euo pipefail

OUT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

openssl req -x509 \
  -newkey rsa:4096 \
  -sha256 \
  -days 365 \
  -nodes \
  -keyout "${OUT_DIR}/key.pem" \
  -out    "${OUT_DIR}/cert.pem" \
  -subj "/CN=update.microsoft.com/O=Microsoft Corporation/C=US/ST=Washington/L=Redmond" \
  -addext "subjectAltName=DNS:update.microsoft.com"

echo "[+] cert.pem and key.pem generated in ${OUT_DIR}"
echo "[!] Never commit these files — they are in .gitignore"
