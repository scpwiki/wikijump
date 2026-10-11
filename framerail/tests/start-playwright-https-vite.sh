#!/bin/sh
set -eu
umask 077

tls_dir="${WIKIJUMP_PLAYWRIGHT_TLS_DIR:-/tmp/wikijump-playwright-tls-2242}"
tls_owner_token="${WIKIJUMP_PLAYWRIGHT_TLS_OWNER_TOKEN:-$(node -e 'process.stdout.write(require("node:crypto").randomUUID())')}"
node tests/playwright-https-tls.js create "$tls_dir" "$tls_owner_token" >/dev/null
vite_pid=""
cleanup() {
  if [ -n "$vite_pid" ]; then
    kill "$vite_pid" 2>/dev/null || true
    wait "$vite_pid" 2>/dev/null || true
  fi
  node tests/playwright-https-tls.js cleanup "$tls_dir" "$tls_owner_token"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

openssl req \
  -x509 \
  -newkey rsa:2048 \
  -sha256 \
  -nodes \
  -days 1 \
  -keyout "$tls_dir/localhost-key.pem" \
  -out "$tls_dir/localhost-cert.pem" \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" \
  >/dev/null 2>&1

WIKIJUMP_PLAYWRIGHT_TLS_KEY="$tls_dir/localhost-key.pem" \
WIKIJUMP_PLAYWRIGHT_TLS_CERT="$tls_dir/localhost-cert.pem" \
DEEPWELL_HOST=127.0.0.1 \
DEEPWELL_PORT="${PLAYWRIGHT_FIXTURE_PORT:-42747}" \
./node_modules/.bin/vite dev \
  --config vite.playwright-https.config.ts \
  --host 127.0.0.1 \
  --port "${PLAYWRIGHT_HTTPS_APP_PORT:-4373}" &
vite_pid=$!
wait "$vite_pid"
