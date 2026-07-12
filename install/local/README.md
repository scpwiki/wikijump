## LOCAL docker-compose

This `docker-compose.yaml` (and corresponding `docker-compose.dev.yaml`) file are used in standing up local instances of Wikijump. The convenience script `./deploy.py` is provided to make management easier, providing options for common variations.

For disposable local bulk-import runs only, `docker-compose.postgres-perf.yaml` can be layered onto the base local compose file to relax Postgres durability settings for faster writes: `docker compose -f docker-compose.yaml -f docker-compose.postgres-perf.yaml up -d`. These settings can corrupt data after a crash and are fenced to `install/local`; do not use the override with persistent, shared, or production databases.

There are two important things to note about the local tier:
1. It runs its containers in "watch mode". This means that building the service takes place after container start, not at container build time, and that if you modify local watched files, the service will rebuild and restart.
2. Which is related to the fact that several directories are instead *mapped* into the container rather than copied into it. This way, any local changes are reflected in the container.

See `docs/development.md` for more information on local deployments.

For prebuilt/no-dev lab runtimes that still contain `cargo-watch`, use
`deepwell_hot_reload.py` to copy a Deepwell candidate into the running
container without rebuilding its image. The guarded workflow and its
limitations are documented in `docs/deepwell-container-hot-reload.md`.

## Local HTTPS certificates

The local Caddy container terminates HTTPS for `*.wikijump.localhost` and `*.wjfiles.localhost` using Caddy's local certificate authority. The Caddy data and config directories are stored in named Docker volumes:

```text
local-caddy-data -> /data
local-caddy-config -> /config
```

These volumes preserve `/data/caddy/pki/authorities/local/root.crt`, so restarting or recreating the local Caddy container does not rotate the local root CA and invalidate a browser trust-store entry.

If your browser reports `net::ERR_CERT_AUTHORITY_INVALID`, export the current Caddy root certificate and trust it in your operating system or browser profile. Use the same Compose project name that started the stack:

```bash
project=wikijump
caddy_container="$(docker compose -p "$project" -f docker-compose.yaml -f docker-compose.dev.yaml ps -q caddy)"
docker cp "$caddy_container:/data/caddy/pki/authorities/local/root.crt" ./caddy-local-root.crt
```

If you used a different project name, replace `wikijump` with that name. You can confirm the resolved Caddy container with:

```bash
docker compose -p "$project" -f docker-compose.yaml -f docker-compose.dev.yaml ps caddy
```

On Windows, import the certificate into the current user's trusted root store:

```powershell
Import-Certificate -FilePath .\caddy-local-root.crt -CertStoreLocation Cert:\CurrentUser\Root
```

On Linux, install it into your distribution's local CA store. For Debian and Ubuntu:

```bash
sudo cp caddy-local-root.crt /usr/local/share/ca-certificates/wikijump-caddy-local-root.crt
sudo update-ca-certificates
```

On macOS, add it to the login keychain and mark it trusted for SSL:

```bash
sudo security add-trusted-cert -d -r trustRoot -k ~/Library/Keychains/login.keychain-db caddy-local-root.crt
```

If Caddy's local CA already rotated before this volume persistence was added, trust the newly exported `root.crt` and reload the affected browser tab. If Chrome still shows the old interstitial, close and reopen that tab or restart Chrome so it refreshes certificate verification state.
