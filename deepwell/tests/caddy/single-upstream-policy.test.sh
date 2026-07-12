#!/bin/sh
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)
network="wikijump-caddy-recovery-$$"
caddy_container="$network-caddy"
upstream_container="$network-upstream"
temporary=$(mktemp -d)

# shellcheck disable=SC2317 # invoked through trap
cleanup() {
  docker rm -f "$caddy_container" "$upstream_container" >/dev/null 2>&1 || true
  docker network rm "$network" >/dev/null 2>&1 || true
  rm -rf "$temporary"
}
trap cleanup EXIT INT TERM

for fixture in \
  "$repo_root/deepwell/tests/caddy/Caddyfile.basic_local" \
  "$repo_root/deepwell/tests/caddy/Caddyfile.basic_localdev"; do
  docker run --rm -v "$fixture:/etc/caddy/Caddyfile:ro" caddy:alpine \
    caddy adapt --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null 2>&1
  docker run --rm -v "$fixture:/etc/caddy/Caddyfile:ro" caddy:alpine \
    caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null 2>&1
done

cat >"$temporary/Caddyfile" <<'EOF'
:80 {
	reverse_proxy recovery-upstream:8080
}
EOF

docker network create "$network" >/dev/null
docker run -d --name "$upstream_container" --network "$network" \
  python:3-alpine sleep infinity >/dev/null
upstream_ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$upstream_container")
sed "s/recovery-upstream/$upstream_ip/" "$temporary/Caddyfile" >"$temporary/Caddyfile.resolved"
mv "$temporary/Caddyfile.resolved" "$temporary/Caddyfile"
docker run -d --name "$caddy_container" --network "$network" \
  -v "$temporary/Caddyfile:/etc/caddy/Caddyfile:ro" caddy:alpine >/dev/null

if docker exec "$caddy_container" wget -qO- http://127.0.0.1/ >/dev/null 2>&1; then
  echo 'request unexpectedly succeeded while the sole upstream was absent' >&2
  exit 1
fi

docker exec -d "$upstream_container" sh -c \
  'mkdir -p /srv && printf recovered >/srv/index.html && cd /srv && python -m http.server 8080'

attempt=0
while [ "$attempt" -lt 20 ]; do
  response=$(docker exec "$caddy_container" wget -qO- http://127.0.0.1/ 2>/dev/null || true)
  [ "$response" = recovered ] && exit 0
  attempt=$((attempt + 1))
  sleep 1
done

echo 'single upstream did not recover after becoming available' >&2
exit 1
