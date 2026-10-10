// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const LOCAL_SITE = "scpaiueouiuiuiui"

let previousWorkingDirectory
let vite
let requirePageMutationUserId
let deepwellRequestHeaders

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()
  ;({ requirePageMutationUserId } = await vite.ssrLoadModule(
    "/src/lib/server/load/page/page-action-context.ts"
  ))
  ;({ deepwellRequestHeaders } = await vite.ssrLoadModule(
    "/src/lib/server/deepwell/index.ts"
  ))
})

after(async () => {
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

test("the local page mutation actor is wired into the trusted Deepwell request", () => {
  const requestContext = { siteId: 17, page: "draft" }
  const context = {
    requestContext,
    sessionUserId: undefined,
    siteId: 17,
    siteSlug: LOCAL_SITE
  }

  assert.equal(requirePageMutationUserId(context, 17), -1)
  assert.equal(requestContext.localPageMutationActor, true)
  assert.equal(
    deepwellRequestHeaders(requestContext, "Bearer trusted-rpc-token")[
      "X-Deepwell-Local-Page-Mutation-Actor"
    ],
    "-1"
  )
})

test("mirrors and cross-site submissions never mark a local Deepwell actor", () => {
  for (const [siteSlug, requestSiteId, mutationSiteId] of [
    ["scp-wiki", 17, 17],
    ["scp-jp", 17, 17],
    [LOCAL_SITE, 17, 18]
  ]) {
    const requestContext = { siteId: requestSiteId }
    const context = {
      requestContext,
      sessionUserId: undefined,
      siteId: requestSiteId,
      siteSlug
    }

    assert.throws(() => requirePageMutationUserId(context, mutationSiteId))
    assert.equal(requestContext.localPageMutationActor, undefined)
    assert.equal(
      "X-Deepwell-Local-Page-Mutation-Actor" in
        deepwellRequestHeaders(requestContext, "Bearer trusted-rpc-token"),
      false
    )
  }
})

test("authenticated mutations keep their session actor and do not set the local marker", () => {
  const requestContext = { siteId: 17, sessionToken: "session-token" }
  const context = {
    requestContext,
    sessionUserId: 42,
    siteId: 17,
    siteSlug: LOCAL_SITE
  }

  assert.equal(requirePageMutationUserId(context, 17), 42)
  assert.equal(requestContext.localPageMutationActor, undefined)
})
