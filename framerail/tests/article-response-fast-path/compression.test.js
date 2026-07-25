import { strict as assert } from "node:assert"
import test from "node:test"

import {
  brotliDecompress,
  createCountingStore,
  createFastPathFixtureStore,
  fastPathHeaders,
  // eslint-disable-next-line no-redeclare
  fetch,
  gunzip,
  largeHtmlBody,
  requestRaw,
  withServer
} from "./helpers.js"

test("article response fast path serves a hot anonymous article hit without calling handler", async () => {
  const stores = await createFastPathFixtureStore()

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 200)
    assert.equal(response.headers.get("content-type"), "text/html; charset=utf-8")
    assert.equal(response.headers.get("x-cache-fixture"), "hit")
    assert.equal(
      await response.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path serves br and gzip hot replay variants", async () => {
  const body = largeHtmlBody()
  const stores = await createFastPathFixtureStore({
    body,
    headers: [
      ["content-type", "text/html; charset=utf-8"],
      ["etag", '"strong-etag"'],
      ["transfer-encoding", "chunked"],
      ["x-cache-fixture", "hit"]
    ]
  })

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const br = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "br, gzip" }
    })
    assert.equal(br.status, 200)
    assert.equal(br.headers["content-encoding"], "br")
    assert.equal(br.headers.vary, "Accept-Encoding")
    assert.equal(br.headers["content-length"], String(br.body.length))
    assert.equal(br.headers["transfer-encoding"], undefined)
    assert.equal(br.headers.etag, undefined)
    assert.equal((await brotliDecompress(br.body)).toString("utf8"), body)

    const gzip = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "gzip" }
    })
    assert.equal(gzip.status, 200)
    assert.equal(gzip.headers["content-encoding"], "gzip")
    assert.equal(gzip.headers.vary, "Accept-Encoding")
    assert.equal(gzip.headers["content-length"], String(gzip.body.length))
    assert.equal((await gunzip(gzip.body)).toString("utf8"), body)
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path preserves weak etags on compressed variants", async () => {
  const body = largeHtmlBody()
  const stores = await createFastPathFixtureStore({
    body,
    headers: [
      ["content-type", "text/html; charset=utf-8"],
      ["etag", 'W/"weak-etag"']
    ]
  })

  await withServer(stores, async ({ baseUrl }) => {
    const response = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "br" }
    })
    assert.equal(response.headers["content-encoding"], "br")
    assert.equal(response.headers.etag, 'W/"weak-etag"')
  })
})

test("article response fast path honors compression q-values", async () => {
  const body = largeHtmlBody()
  const stores = await createFastPathFixtureStore({ body })

  await withServer(stores, async ({ baseUrl }) => {
    const gzip = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "br;q=0, gzip;q=1" }
    })
    assert.equal(gzip.headers["content-encoding"], "gzip")
    assert.equal((await gunzip(gzip.body)).toString("utf8"), body)

    const identity = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "gzip;q=0, br;q=0" }
    })
    assert.equal(identity.headers["content-encoding"], undefined)
    assert.equal(identity.headers.vary, "Accept-Encoding")
    assert.equal(identity.body.toString("utf8"), body)
  })
})

test("article response fast path identity replay varies when compressed variants exist", async () => {
  const body = largeHtmlBody()
  const stores = await createFastPathFixtureStore({ body })

  await withServer(stores, async ({ baseUrl }) => {
    const response = await requestRaw(`${baseUrl}/scp-173`, {
      headers: fastPathHeaders
    })
    assert.equal(response.status, 200)
    assert.equal(response.headers["content-encoding"], undefined)
    assert.equal(response.headers.vary, "Accept-Encoding")
    assert.equal(response.headers["content-length"], undefined)
    assert.equal(response.body.toString("utf8"), body)
  })
})

test("article response fast path HEAD chooses compressed headers and sends no body", async () => {
  const stores = await createFastPathFixtureStore({ body: largeHtmlBody() })

  await withServer(stores, async ({ baseUrl }) => {
    const response = await requestRaw(`${baseUrl}/scp-173`, {
      method: "HEAD",
      headers: { ...fastPathHeaders, "accept-encoding": "br" }
    })
    assert.equal(response.status, 200)
    assert.equal(response.headers["content-encoding"], "br")
    assert.equal(response.headers.vary, "Accept-Encoding")
    assert.equal(Number.parseInt(response.headers["content-length"], 10) > 0, true)
    assert.equal(response.body.length, 0)
  })
})

test("article response fast path reuses one hot token for identity br and gzip", async () => {
  const body = largeHtmlBody()
  const stores = await createFastPathFixtureStore({ body })
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)

  await withServer({ responseStore, tokenStore }, async ({ baseUrl, handlerCalls }) => {
    const identity = await requestRaw(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(identity.body.toString("utf8"), body)

    const br = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "br" }
    })
    assert.equal((await brotliDecompress(br.body)).toString("utf8"), body)

    const gzip = await requestRaw(`${baseUrl}/scp-173`, {
      headers: { ...fastPathHeaders, "accept-encoding": "gzip" }
    })
    assert.equal((await gunzip(gzip.body)).toString("utf8"), body)
    assert.equal(handlerCalls(), 0)
    assert.equal(tokenStore.getCalls(), 1)
    assert.equal(responseStore.getCalls(), 1)
  })
})

test("article response fast path does not compress ineligible replay responses", async () => {
  const cases = [
    { body: "<!doctype html><p>small</p>", contentEncoding: undefined },
    {
      headers: [["content-type", "application/json"]],
      body: largeHtmlBody(),
      contentEncoding: undefined
    },
    {
      headers: [
        ["content-type", "text/html"],
        ["content-encoding", "gzip"]
      ],
      body: largeHtmlBody(),
      contentEncoding: "gzip"
    },
    {
      headers: [
        ["content-type", "text/html"],
        ["cache-control", "public, no-transform"]
      ],
      body: largeHtmlBody(),
      contentEncoding: undefined
    },
    {
      headers: [
        ["content-type", "text/html"],
        ["set-cookie", "wikijump_token=session"]
      ],
      body: largeHtmlBody(),
      contentEncoding: undefined
    }
  ]

  for (const candidate of cases) {
    const stores = await createFastPathFixtureStore(candidate)
    await withServer(stores, async ({ baseUrl }) => {
      const response = await requestRaw(`${baseUrl}/scp-173`, {
        headers: { ...fastPathHeaders, "accept-encoding": "br, gzip" }
      })
      assert.equal(response.headers["content-encoding"], candidate.contentEncoding)
      assert.equal(response.body.toString("utf8"), candidate.body)
    })
  }
})
