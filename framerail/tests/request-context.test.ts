import assert from "node:assert/strict"
import test from "node:test"

import { withDefaultPageContext } from "../src/lib/server/request-context.ts"

test("adds the server-resolved default page to a root request context", () => {
  assert.deepEqual(
    withDefaultPageContext({ sessionToken: "session", siteId: 7 }, "start"),
    {
      sessionToken: "session",
      siteId: 7,
      page: "start"
    }
  )
})

test("creates a page context when no request context was stored", () => {
  assert.deepEqual(withDefaultPageContext(undefined, "start"), { page: "start" })
})

test("does not replace the route-derived page", () => {
  const requestContext = { sessionToken: "session", siteId: 7, page: "requested" }

  assert.equal(withDefaultPageContext(requestContext, "start"), requestContext)
})

test("does not add an empty default page", () => {
  const requestContext = { sessionToken: "session", siteId: 7 }

  assert.equal(withDefaultPageContext(requestContext, ""), requestContext)
})
