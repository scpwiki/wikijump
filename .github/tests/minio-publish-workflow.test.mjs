import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..")
const source = readFileSync(
  path.join(root, ".github/workflows/docker-push-minio.yaml"),
  "utf8"
)

test("Minio publishing is triggered only by relevant develop pushes", () => {
  const trigger = source.slice(source.indexOf("on:\n"), source.indexOf("\nenv:\n"))

  assert.match(trigger, /^\s*push:$/m)
  assert.match(trigger, /^\s*branches:\s*\n\s*- develop$/m)
  assert.match(trigger, /- 'install\/local\/minio\/\*'/)
  assert.match(trigger, /- '\.github\/workflows\/docker-push-minio\.yaml'/)
  assert.doesNotMatch(trigger, /pull_request|workflow_dispatch|branches-ignore/)
  assert.equal((trigger.match(/^\s*- develop$/gm) ?? []).length, 1)
})

test("Minio publish namespace and develop guards retain their audited intent", () => {
  assert.match(source, /^\s*REGISTRY: ghcr\.io$/m)
  assert.match(source, /^\s*IMAGE_NAME: scpwiki\/wikijump$/m)
  assert.match(source, /^\s*TAG: minio$/m)
  assert.match(source, /images: \$\{\{ env\.REGISTRY \}\}\/\$\{\{ env\.IMAGE_NAME \}\}/)
  assert.match(source, /value=\$\{\{ env\.TAG \}\}/)
  assert.match(source, /push: \$\{\{ github\.ref == 'refs\/heads\/develop' \}\}/)
  assert.match(source, /^\s*packages: write$/m)
  assert.match(source, /^\s*attestations: write$/m)
  assert.match(source, /^\s*id-token: write$/m)
})

test("third-party Minio publishing actions are pinned to full commits", () => {
  const uses = [...source.matchAll(/^\s*uses:\s*([^\s#]+)/gm)].map((match) => match[1])
  assert.ok(uses.length >= 4)

  for (const action of uses) {
    assert.match(action, /^[^@]+@[0-9a-f]{40}$/, action)
  }
})
