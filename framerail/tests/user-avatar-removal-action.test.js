// @ts-nocheck
import { strict as assert } from "node:assert"
import { fileURLToPath } from "node:url"
import test from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))

/**
 * Build the multipart request the profile editor submits. Superforms reads
 * the submitted values from `__superform_json`; a real file is restored
 * from its `__superform_file_avatar` entry, while an empty file input
 * submits no file.
 */
const editRequest = ({ json, file }) => {
  const data = new FormData()
  data.set("__superform_json", JSON.stringify(json))
  data.set("__superform_id", "avatar-removal-test")
  if (file) data.set("__superform_file_avatar", file, file.name)
  return new Request("https://wikijump.test/-/user?/userEdit", {
    method: "POST",
    body: data
  })
}

const eventFor = (request) => ({
  request,
  cookies: {
    get(name) {
      assert.equal(name, "wikijump_token")
      return "avatar-removal-session"
    }
  },
  getClientAddress: () => "192.0.2.64",
  locals: { requestContext: { sessionToken: "avatar-removal-session", siteId: 17 } }
})

const PNG_FILE = () =>
  new File([new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])], "new.png", {
    type: "image/png"
  })

test("only an explicit boolean removal flag clears the stored avatar", async () => {
  const previousWorkingDirectory = process.cwd()
  let vite
  let client
  let originalRequest

  try {
    process.chdir(root)
    vite = await createTestViteServer()
    ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
    const { userEditAction } = await vite.ssrLoadModule("/src/lib/server/load/user.ts")
    originalRequest = client.request

    const calls = []
    client.request = async (method, params, context) => {
      calls.push({ method, params, context })
      if (method === "session_get") {
        return {
          session_token: "avatar-removal-session",
          user_id: 6000008,
          created_at: "2026-10-10T00:00:00Z",
          expires_at: "2099-01-01T00:00:00Z",
          ip_address: "192.0.2.64",
          user_agent: "avatar removal test",
          restricted: false
        }
      }
      if (method === "user_edit") return { user_id: params.user }
      throw new Error(`Unexpected Deepwell method ${method}`)
    }
    const userEdits = () => calls.filter(({ method }) => method === "user_edit")
    const reset = () => (calls.length = 0)

    // 1. Omitted flag and an empty file input never touch the association.
    await userEditAction(eventFor(editRequest({ json: [{ avatar: -1 }] })))
    assert.equal(userEdits().length, 1)
    assert.equal("avatar_uploaded_blob_id" in userEdits()[0].params, false)
    reset()

    // 2. The explicit boolean flag clears the association for the session user.
    const removed = await userEditAction(
      eventFor(editRequest({ json: [{ avatar: -1, removeAvatar: 1 }, true] }))
    )
    assert.equal(removed.status, undefined)
    assert.equal(userEdits().length, 1)
    assert.equal(userEdits()[0].params.user, 6000008)
    assert.equal(userEdits()[0].params.avatar_uploaded_blob_id, null)
    reset()

    // 3. A flag that is not a literal boolean is rejected before any mutation.
    for (const flag of ["true", null, 1]) {
      const rejected = await userEditAction(
        eventFor(editRequest({ json: [{ avatar: -1, removeAvatar: 1 }, flag] }))
      )
      assert.equal(rejected.status, 400, `flag ${JSON.stringify(flag)} must be rejected`)
      assert.equal(userEdits().length, 0)
    }

    // 4. A removal flag never accompanies a real new image.
    const conflict = await userEditAction(
      eventFor(
        editRequest({
          json: [{ avatar: 1, removeAvatar: 2 }, "avatar", true],
          file: PNG_FILE()
        })
      )
    )
    assert.equal(conflict.status, 400)
    assert.match(conflict.data.message, /not both/u)
    assert.equal(userEdits().length, 0)
  } finally {
    if (client && originalRequest) client.request = originalRequest
    if (vite) await vite.close()
    process.chdir(previousWorkingDirectory)
  }
})
