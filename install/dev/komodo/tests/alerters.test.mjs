import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { readFile, writeFile } from "node:fs/promises"
import { mkdtemp, rm } from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const komodoRoot = fileURLToPath(new URL("..", import.meta.url))
const alertersPath = path.join(komodoRoot, "alerters.toml")

const embeddedCompose = (source) => {
  const match = source.match(/file_contents = """\n([\s\S]*?)\n"""/u)
  assert.ok(match, "alerters.toml must contain an embedded Compose document")
  return `${match[1]}\n`
}

test("embedded alerter Compose config is valid and internal-only", async (t) => {
  const source = await readFile(alertersPath, "utf8")
  const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), "wikijump-alerter-compose-"))
  t.after(() => rm(temporaryRoot, { recursive: true, force: true }))
  const composePath = path.join(temporaryRoot, "compose.yaml")
  await writeFile(composePath, embeddedCompose(source))
  await writeFile(path.join(temporaryRoot, ".env"), "")

  const result = spawnSync(
    "docker",
    ["compose", "-f", composePath, "config", "--format", "json"],
    { encoding: "utf8" }
  )
  assert.equal(result.status, 0, result.stderr)

  const config = JSON.parse(result.stdout)
  const alerter = config.services.alerter
  assert.deepEqual(alerter.expose, ["7000"])
  assert.equal(alerter.ports, undefined)
  assert.equal(alerter.extra_hosts, undefined)
  assert.deepEqual(Object.keys(alerter.networks), ["komodo"])
  assert.equal(config.networks.komodo.external, true)
  assert.equal(config.networks.komodo.name, "komodo_default")
})

test("Komodo reaches the alerter only through its Compose service name", async () => {
  const source = await readFile(alertersPath, "utf8")

  assert.match(source, /endpoint\.params\.url = "http:\/\/alerter:7000"/u)
  assert.doesNotMatch(source, /host\.docker\.internal|7100:7000/u)
})
