import assert from "node:assert/strict"
import test from "node:test"

import {
  handleCachedRequest,
  readCachedResponseWithFallback
} from "../src/lib/server/cache/request-orchestration.ts"
import { executePageAction } from "../src/lib/server/load/page/page-action-execution.ts"

test("request cache hits receive security headers without resolving the app", async () => {
  const calls: string[] = []
  const cachedResponse = new Response("cached")

  const response = await handleCachedRequest({
    readCachedResponse: async () => {
      calls.push("cache")
      return cachedResponse
    },
    resolveResponse: async () => {
      calls.push("resolve")
      return new Response("resolved")
    },
    applySecurityHeaders: (value) => {
      calls.push("security")
      value.headers.set("x-content-type-options", "nosniff")
    },
    writeResolvedResponse: async () => {
      calls.push("write")
    }
  })

  assert.equal(response, cachedResponse)
  assert.equal(await response.text(), "cached")
  assert.equal(response.headers.get("x-content-type-options"), "nosniff")
  assert.deepEqual(calls, ["cache", "security"])
})

test("request cache lookup falls back to Deepwell metadata", async () => {
  const calls: string[] = []
  const deepwellResponse = new Response("deepwell")

  const response = await readCachedResponseWithFallback({
    readPrimary: async () => {
      calls.push("token")
      throw new Error("token cache unavailable")
    },
    readFallback: async () => {
      calls.push("deepwell")
      return deepwellResponse
    }
  })

  assert.equal(response, deepwellResponse)
  assert.deepEqual(calls, ["token", "deepwell"])
})

test("cache misses resolve the app, apply headers, and write the response", async () => {
  const calls: string[] = []

  const response = await handleCachedRequest({
    readCachedResponse: async () => {
      calls.push("cache")
      return null
    },
    resolveResponse: async () => {
      calls.push("resolve")
      return new Response("resolved")
    },
    applySecurityHeaders: () => {
      calls.push("security")
    },
    writeResolvedResponse: async () => {
      calls.push("write")
    }
  })

  assert.equal(await response.text(), "resolved")
  assert.deepEqual(calls, ["cache", "resolve", "security", "write"])
})

test("representative page mutations preserve success and failure boundaries", async () => {
  const mutations: { pageId: number; value: number }[] = []
  const success = await executePageAction(
    async () => {
      const input = { pageId: 17, value: 1 }
      mutations.push(input)
      return { accepted: true }
    },
    () => ({ status: 500 })
  )
  const failure = await executePageAction(
    async () => {
      throw new Error("permission denied")
    },
    (error) => ({
      status: 403,
      message: error instanceof Error ? error.message : "unexpected error"
    })
  )

  assert.deepEqual(mutations, [{ pageId: 17, value: 1 }])
  assert.deepEqual(success, { res: { accepted: true } })
  assert.deepEqual(failure, { status: 403, message: "permission denied" })
})
