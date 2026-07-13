#!/usr/bin/env bash
set -Eeuo pipefail

repo=${WIKIJUMP_CODEX_REPO:-/workspace/wikijump}
script_revision=2026-07-14.1
required_node_major=24
required_rust_version=1.95.0
pnpm_version=11.12.0
legacy_node24_link=/opt/wikijump/node24
node24_env=/root/.config/wikijump/node24.sh
cargo_command=(rustup run "$required_rust_version" cargo)

printf 'Wikijump Codex Cloud setup revision %s\n' "$script_revision"

retry() {
  local attempt=1
  local max_attempts=5
  local delay=2
  local status

  until "$@"; do
    status=$?
    if (( attempt >= max_attempts )); then
      printf 'Command failed after %d attempts (exit %d):' "$attempt" "$status" >&2
      printf ' %q' "$@" >&2
      printf '\n' >&2
      return "$status"
    fi

    printf 'Command failed (attempt %d/%d); retrying in %ds:' \
      "$attempt" "$max_attempts" "$delay" >&2
    printf ' %q' "$@" >&2
    printf '\n' >&2
    sleep "$delay"
    attempt=$((attempt + 1))
    delay=$((delay * 2))
  done
}

download_verified() {
  local url=$1
  local expected_sha256=$2
  local destination=$3
  local temporary

  temporary=$(mktemp)
  retry curl --fail --location --silent --show-error \
    --connect-timeout 20 --max-time 300 \
    --output "$temporary" "$url"
  printf '%s  %s\n' "$expected_sha256" "$temporary" | sha256sum -c -
  sudo install -m 0755 "$temporary" "$destination"
  rm -f "$temporary"
}

strip_path_entry() {
  local entry_to_remove=$1
  local current_entry
  local new_path=
  local first=1
  local -a path_entries=()

  IFS=: read -r -a path_entries <<<"${PATH-}"
  for current_entry in "${path_entries[@]}"; do
    [[ "$current_entry" == "$entry_to_remove" ]] && continue
    if (( first )); then
      new_path=$current_entry
      first=0
    else
      new_path+=":${current_entry}"
    fi
  done

  PATH=$new_path
  export PATH
  hash -r
}

cleanup_legacy_node24_link() {
  # Earlier revisions used this stable symlink. On a cached maintenance run,
  # command -v resolved through the symlink and the script replaced it with a
  # self-referential link. Remove it before NVM or Node executes, and remove its
  # bin directory from the current shell's PATH.
  if [[ -L "$legacy_node24_link" ]]; then
    sudo rm -f -- "$legacy_node24_link"
  fi
  strip_path_entry "$legacy_node24_link/bin"
}

install_shell_hooks() {
  local file=$1
  local pre_marker='# >>> wikijump Codex Node 24 (pre) >>>'
  local post_marker='# >>> wikijump Codex Node 24 (post) >>>'
  local source_line='[ -r /root/.config/wikijump/node24.sh ] && . /root/.config/wikijump/node24.sh'
  local temporary

  touch "$file"

  if ! grep -Fqx "$pre_marker" "$file"; then
    temporary=$(mktemp)
    {
      printf '%s\n' "$pre_marker"
      printf '%s\n' "$source_line"
      printf '%s\n' '# <<< wikijump Codex Node 24 (pre) <<<'
      cat "$file"
    } >"$temporary"
    chmod --reference="$file" "$temporary"
    mv "$temporary" "$file"
  fi

  if ! grep -Fqx "$post_marker" "$file"; then
    {
      printf '\n%s\n' "$post_marker"
      printf '%s\n' "$source_line"
      printf '%s\n' '# <<< wikijump Codex Node 24 (post) <<<'
    } >>"$file"
  fi
}

activate_node24() {
  local installed_version
  local actual_node_major
  local git_exclude
  local node_executable
  local node_bin

  cleanup_legacy_node24_link

  export NVM_DIR="${NVM_DIR:-/root/.nvm}"
  if [[ ! -s "$NVM_DIR/nvm.sh" ]]; then
    printf 'NVM initialization script is missing: %s\n' "$NVM_DIR/nvm.sh" >&2
    return 1
  fi

  set +u
  # shellcheck disable=SC1091
  . "$NVM_DIR/nvm.sh"

  installed_version=$(nvm version "$required_node_major" 2>/dev/null || true)
  if [[ -z "$installed_version" || "$installed_version" == N/A ]]; then
    if ! retry nvm install "$required_node_major"; then
      set -u
      return 1
    fi
  fi

  nvm use --silent "$required_node_major"
  nvm alias default "$required_node_major" >/dev/null
  node_executable=$(nvm which "$required_node_major" 2>/dev/null || true)
  set -u

  if [[ -z "$node_executable" || ! -x "$node_executable" ]]; then
    printf 'NVM did not provide an executable for Node.js %s.\n' \
      "$required_node_major" >&2
    return 1
  fi

  actual_node_major=$("$node_executable" -p 'process.versions.node.split(".")[0]')
  if [[ "$actual_node_major" != "$required_node_major" ]]; then
    printf 'Failed to activate Node.js %s; Node.js %s is active.\n' \
      "$required_node_major" "$actual_node_major" >&2
    return 1
  fi

  node_bin=$(dirname "$node_executable")
  case "${PATH-}" in
    "$node_bin"|"$node_bin":*) ;;
    *) PATH="$node_bin${PATH:+:${PATH}}" ;;
  esac
  export PATH
  hash -r

  mkdir -p "$(dirname "$node24_env")"
  {
    printf '_wikijump_node24_bin=%q\n' "$node_bin"
    cat <<'NODE_ENV'
case "${PATH-}" in
  "${_wikijump_node24_bin}"|"${_wikijump_node24_bin}":*) ;;
  *) PATH="${_wikijump_node24_bin}${PATH:+:${PATH}}" ;;
esac
export PATH
unset _wikijump_node24_bin
NODE_ENV
  } >"$node24_env"
  chmod 0644 "$node24_env"

  sudo tee /etc/profile.d/wikijump-node24.sh >/dev/null <<'PROFILE_ENV'
[ -r /root/.config/wikijump/node24.sh ] && . /root/.config/wikijump/node24.sh
PROFILE_ENV
  sudo chmod 0644 /etc/profile.d/wikijump-node24.sh

  install_shell_hooks /root/.bashrc
  install_shell_hooks /root/.profile
  if [[ -e /root/.bash_profile ]]; then
    install_shell_hooks /root/.bash_profile
  fi

  if ! git -C "$repo" ls-files --error-unmatch .nvmrc >/dev/null 2>&1; then
    printf '%s\n' "$required_node_major" >"$repo/.nvmrc"
    git_exclude=$(git -C "$repo" rev-parse --path-format=absolute --git-path info/exclude)
    mkdir -p "$(dirname "$git_exclude")"
    if ! grep -Fqx '/.nvmrc' "$git_exclude" 2>/dev/null; then
      printf '%s\n' '/.nvmrc' >>"$git_exclude"
    fi
  fi
}

if [[ "$repo" != /* || ( ! -d "$repo/.git" && ! -f "$repo/.git" ) ]]; then
  printf 'Wikijump repository not found at %s. Set WIKIJUMP_CODEX_REPO to its absolute path.\n' "$repo" >&2
  exit 1
fi

cd "$repo"

retry sudo apt-get update
retry sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  build-essential ca-certificates clang cmake curl docker.io docker-compose-v2 jq \
  libmagic-dev libssl-dev pkg-config postgresql postgresql-client \
  python3-dev python3-venv python3-yaml redis-server shellcheck

activate_node24
printf 'Using %s at %s\n' "$(node --version)" "$(command -v node)"

retry env RUSTUP_MAX_RETRIES=5 rustup toolchain install "$required_rust_version" \
  --profile minimal --component clippy,rustfmt,rust-src --no-self-update
printf 'Using %s\n' "$(rustup run "$required_rust_version" rustc --version)"

export npm_config_fetch_retries=5
export npm_config_fetch_retry_factor=2
export npm_config_fetch_retry_mintimeout=10000
export npm_config_fetch_retry_maxtimeout=120000

installed_pnpm=$(pnpm --version 2>/dev/null || true)
if [[ "$installed_pnpm" != "$pnpm_version" ]]; then
  # Remove a conflicting Corepack shim only when replacement is necessary.
  # Leaving a verified npm-installed pnpm in place keeps maintenance idempotent.
  corepack disable pnpm >/dev/null 2>&1 || true
  retry npm install --global --no-audit --no-fund "pnpm@${pnpm_version}"
  hash -r
fi
[[ "$(pnpm --version)" == "$pnpm_version" ]]
printf 'Using pnpm %s at %s\n' "$(pnpm --version)" "$(command -v pnpm)"

export UV_HTTP_TIMEOUT=120
retry uv pip install --system -r deepwell/importer/requirements.txt
retry pnpm --dir framerail fetch --ignore-scripts --frozen-lockfile
retry pnpm --dir install/local/wikidot-verification fetch --ignore-scripts --frozen-lockfile
retry pnpm --dir locales/typed fetch --ignore-scripts --frozen-lockfile

export CARGO_NET_RETRY=5
export CARGO_HTTP_TIMEOUT=120
retry "${cargo_command[@]}" fetch --locked --manifest-path deepwell/Cargo.toml
retry "${cargo_command[@]}" fetch --locked --manifest-path wws/Cargo.toml
retry "${cargo_command[@]}" fetch --locked --manifest-path locales/validator/Cargo.toml
retry env RUSTFLAGS= "${cargo_command[@]}" install cargo-machete --version 0.9.1 --locked
retry env RUSTFLAGS= "${cargo_command[@]}" install sqlx-cli --version 0.8.6 --locked \
  --no-default-features --features rustls,postgres

rm -rf /tmp/wikijump-go-bin
mkdir -p /tmp/wikijump-go-bin
retry env GOBIN=/tmp/wikijump-go-bin \
  go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.12
sudo install -m 0755 /tmp/wikijump-go-bin/actionlint /usr/local/bin/actionlint

download_verified \
  'https://dl.min.io/server/minio/release/linux-amd64/archive/minio.RELEASE.2025-09-07T16-13-09Z' \
  '7c5bd8512c6e966455b1d198209358b2d191c77a83ab377c4073281065fb855f' \
  /usr/local/bin/minio
download_verified \
  'https://dl.min.io/client/mc/release/linux-amd64/archive/mc.RELEASE.2025-08-13T08-35-41Z' \
  '01f866e9c5f9b87c2b09116fa5d7c06695b106242d829a8bb32990c00312e891' \
  /usr/local/bin/mc

sudo tee /usr/local/bin/wikijump-cloud-services >/dev/null <<'SERVICES'
#!/usr/bin/env bash
set -Eeuo pipefail

sudo service postgresql start
sudo service redis-server start

for _ in $(seq 1 30); do
  pg_isready -h 127.0.0.1 -U postgres >/dev/null 2>&1 && break
  sleep 1
done
if ! pg_isready -h 127.0.0.1 -U postgres >/dev/null 2>&1; then
  echo 'PostgreSQL did not become ready.' >&2
  exit 1
fi

for _ in $(seq 1 30); do
  redis-cli -h 127.0.0.1 ping 2>/dev/null | grep -qx PONG && break
  sleep 1
done
if ! redis-cli -h 127.0.0.1 ping 2>/dev/null | grep -qx PONG; then
  echo 'Redis did not become ready.' >&2
  exit 1
fi

sudo -u postgres psql -v ON_ERROR_STOP=1 \
  -c "ALTER USER postgres PASSWORD 'postgres'" >/dev/null
sudo -u postgres dropdb --if-exists --force wikijump_codex
sudo -u postgres createdb wikijump_codex
redis-cli -h 127.0.0.1 FLUSHALL >/dev/null

if [[ -f /tmp/wikijump-minio.pid ]]; then
  old_pid=$(cat /tmp/wikijump-minio.pid 2>/dev/null || true)
  if [[ "$old_pid" =~ ^[0-9]+$ ]] && kill -0 "$old_pid" 2>/dev/null; then
    if [[ "$(ps -p "$old_pid" -o comm= 2>/dev/null | tr -d ' ')" == minio ]]; then
      kill "$old_pid" 2>/dev/null || true
      for _ in $(seq 1 20); do
        kill -0 "$old_pid" 2>/dev/null || break
        sleep 0.25
      done
    fi
  fi
fi
rm -f /tmp/wikijump-minio.pid
rm -rf /tmp/wikijump-minio-data
mkdir -p /tmp/wikijump-minio-data

MINIO_ROOT_USER=minio \
MINIO_ROOT_PASSWORD=minio-codex-test \
MINIO_REGION_NAME=test \
  nohup minio server /tmp/wikijump-minio-data \
    --address :9000 --console-address :9001 \
    >/tmp/wikijump-minio.log 2>&1 &
echo $! >/tmp/wikijump-minio.pid

for _ in $(seq 1 30); do
  curl -fsS http://127.0.0.1:9000/minio/health/ready >/dev/null 2>&1 && break
  sleep 1
done
if ! curl -fsS http://127.0.0.1:9000/minio/health/ready >/dev/null 2>&1; then
  echo 'MinIO did not become ready. Last log lines:' >&2
  tail -n 100 /tmp/wikijump-minio.log >&2 || true
  exit 1
fi

mc alias set local http://127.0.0.1:9000 minio minio-codex-test >/dev/null
mc mb --ignore-existing local/deepwell-files local/deepwell-text-blocks >/dev/null
SERVICES
sudo chmod 0755 /usr/local/bin/wikijump-cloud-services

wikijump-cloud-services
