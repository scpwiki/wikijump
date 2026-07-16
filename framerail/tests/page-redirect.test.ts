import assert from "node:assert/strict"
import test from "node:test"

import { resolvePageRedirect } from "../src/lib/server/page-redirect.ts"

test("Wikidot module redirects preserve relative query and fragment", () => {
  assert.deepEqual(
    resolvePageRedirect(
      {
        redirect_page: "/target?view=full#history",
        redirect_kind: "wikidot_module"
      },
      "source",
      undefined,
      "https://example.test/source"
    ),
    { status: 301, location: "/target?view=full#history" }
  )
})

test("Wikidot module redirects preserve evidenced absolute HTTP locations", () => {
  assert.deepEqual(
    resolvePageRedirect(
      {
        redirect_page: "http://www.scp-wiki.net/SCP-4000?view=1#part",
        redirect_kind: "wikidot_module"
      },
      "source",
      undefined,
      "https://scp-wiki.wikidot.com/source"
    ),
    {
      status: 301,
      location: "http://www.scp-wiki.net/SCP-4000?view=1#part"
    }
  )
})

test("GET and HEAD receive the same 301 and Location contract", () => {
  for (const method of ["GET", "HEAD"]) {
    const request = new Request("https://example.test/source", { method })
    assert.deepEqual(
      resolvePageRedirect(
        { redirect_page: "/target", redirect_kind: "wikidot_module" },
        "source",
        undefined,
        request.url
      ),
      { status: 301, location: "/target" },
      method
    )
  }
})

test("self redirects fail closed even when a fragment is present", () => {
  for (const location of [
    "/source",
    "/source#again",
    "/%73ource",
    "https://example.test/source",
    "https://example.test/source#again",
    "http://example.test/source",
    "https://example.test:8443/source"
  ]) {
    assert.equal(
      resolvePageRedirect(
        { redirect_page: location, redirect_kind: "wikidot_module" },
        "source",
        undefined,
        "https://example.test/source"
      ),
      null,
      location
    )
  }
})

test("unsupported runtime redirect locations fail closed", () => {
  for (const location of ["target", "//evil.test/path", "javascript:alert(1)"]) {
    assert.equal(
      resolvePageRedirect(
        { redirect_page: location, redirect_kind: "wikidot_module" },
        "source",
        undefined,
        "https://example.test/source"
      ),
      null,
      location
    )
  }
})

test("legacy slug normalization remains a 308 and retains route options", () => {
  assert.deepEqual(
    resolvePageRedirect(
      { redirect_page: "normalized", redirect_kind: null },
      "source",
      "history",
      "https://example.test/source/history"
    ),
    { status: 308, location: "/normalized/history" }
  )
})
