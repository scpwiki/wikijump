import { strict as assert } from "node:assert"
import { readFile } from "node:fs/promises"
import test from "node:test"

const pageFileSourceUrl = new URL(
  "../src/lib/server/deepwell/pageFile.ts",
  import.meta.url
)
const pageActionsSourceUrl = new URL("../src/lib/server/load/page.ts", import.meta.url)

/**
 * @param {string} source
 * @param {string} name
 * @param {string | null} nextName
 */
const exportedFunction = (source, name, nextName) => {
  const start = source.indexOf(`export async function ${name}(`)
  assert.notEqual(start, -1, name)
  const end = nextName
    ? source.indexOf(`export async function ${nextName}(`, start)
    : source.length
  if (nextName) assert.notEqual(end, -1, nextName)
  return source.slice(start, end)
}

test("file mutation RPC serializers include the supplied client IP", async () => {
  const source = await readFile(pageFileSourceUrl, "utf8")
  const cases = [
    ["pageFileCreate", "pageFileDelete", "file_create"],
    ["pageFileEdit", "pageFileMove", "file_edit"],
    ["pageFileRestore", "pageFileHistory", "file_restore"],
    ["pageFileRollback", "pageFileRevision", "file_rollback"]
  ]

  for (const [name, nextName, method] of cases) {
    const body = exportedFunction(source, name, nextName)
    assert.match(body, /ipAddress: string/u, `${name} parameter`)
    assert.match(body, new RegExp(`["']${method}["']`, "u"), `${name} RPC`)
    assert.match(body, /ip_address:\s*ipAddress/u, `${name} serialization`)
  }
})

test("file mutation actions read and forward the server client address", async () => {
  const source = await readFile(pageActionsSourceUrl, "utf8")
  const cases = [
    ["pageFileUploadAction", "pageFileDeleteAction", "pageFileCreate"],
    ["pageFileEditAction", "pageFileMoveAction", "pageFileEdit"],
    ["pageFileRestoreAction", "pageFileHistoryAction", "pageFileRestore"],
    ["pageFileRollbackAction", null, "pageFileRollback"]
  ]

  for (const [name, nextName, callee] of cases) {
    const body = exportedFunction(source, name, nextName)
    assert.match(body, /getClientAddress\s*\}/u, `${name} request event`)
    assert.match(
      body,
      new RegExp(`await ${callee}\\([\\s\\S]*?getClientAddress\\(\\)`, "u")
    )
    assert.match(
      body,
      /getClientAddress\(\),\s*\{\s*sessionToken,\s*siteId,\s*page:\s*pageId\s*\}/u,
      `${name} argument order`
    )
  }
})
