#!/bin/sh
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)
compose_file="$repo_root/install/dev/docker-compose.yaml"
project="wikijump-valkey-auth-test-$$"
password='CodexValkey_420-safe'

cleanup() {
  VALKEY_PASSWORD="$password" docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
  VALKEY_PASSWORD='unsafe@password420' docker compose -p "$project-unsafe" -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

if env -u VALKEY_PASSWORD docker compose -p "$project" -f "$compose_file" config >/dev/null 2>&1; then
  echo 'compose config unexpectedly accepted a missing VALKEY_PASSWORD' >&2
  exit 1
fi

config=$(VALKEY_PASSWORD="$password" docker compose -p "$project" -f "$compose_file" config)
printf '%s\n' "$config" | grep -q 'VALKEYCLI_AUTH:'
if printf '%s\n' "$config" | grep -q 'published: "6379"'; then
  echo 'Valkey must not publish port 6379 to the host' >&2
  exit 1
fi
if grep -Eq 'valkey-cli[[:space:]]+(-a|--pass)' "$compose_file"; then
  echo 'Valkey CLI password must not be passed in process arguments' >&2
  exit 1
fi

VALKEY_PASSWORD="$password" docker compose -p "$project" -f "$compose_file" up -d --build cache
VALKEY_PASSWORD="$password" docker compose -p "$project" -f "$compose_file" exec -T cache valkey-cli ping | grep -qx PONG
unauthenticated=$(VALKEY_PASSWORD="$password" docker compose -p "$project" -f "$compose_file" exec -T -e VALKEYCLI_AUTH= cache valkey-cli ping 2>&1 || true)
if [ "$unauthenticated" = PONG ]; then
  echo 'Valkey unexpectedly accepted an unauthenticated request' >&2
  exit 1
fi

if VALKEY_PASSWORD='unsafe@password420' docker compose -p "$project-unsafe" -f "$compose_file" up --build --abort-on-container-exit cache; then
  echo 'Valkey unexpectedly accepted a non-URL-safe password' >&2
  exit 1
fi
