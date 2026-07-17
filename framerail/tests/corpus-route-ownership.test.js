import assert from "node:assert/strict"
import fs from "node:fs/promises"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const ROUTES = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../src/routes")

async function exists(relativePath) {
  try {
    await fs.access(path.join(ROUTES, relativePath))
    return true
  } catch (error) {
    if (error.code === "ENOENT") return false
    throw error
  }
}

test("corpus page slugs own the root about and forum routes", async () => {
  assert.equal(await exists("about/+page.svelte"), false)
  assert.equal(await exists("about/+page.server.ts"), false)
  assert.equal(await exists("forum/+page.svelte"), false)
  assert.equal(await exists("forum/[...fallback]/+page.svelte"), false)
  assert.equal(await exists("forum/[fallback]/[...extra]/+page.svelte"), true)
  assert.equal(await exists("[slug]/[...extra]/+page.svelte"), true)
})

test("platform information remains available under the reserved application prefix", async () => {
  assert.equal(await exists("[x+2d]/about/+page.svelte"), true)
  assert.equal(await exists("[x+2d]/about/+page.server.ts"), true)
})
