import { strict as assert } from "node:assert"
import test from "node:test"

import {
  resolveWikidotSessionUserName,
  resolveWikidotSiteTagline,
  resolveWikidotSiteTitle,
  shouldUseSandboxWikidotChrome
} from "../src/lib/wikidot/wikidot-chrome.js"

test("selects sandbox Wikidot chrome for imported sandbox source pages", () => {
  assert.equal(
    shouldUseSandboxWikidotChrome({
      site: { slug: "sandbox-for-codex", from_wikidot: false },
      page: { from_wikidot: true },
      page_revision: { from_wikidot: true },
      wikidot_snapshot: { source_site: "sandbox-for-codex" }
    }),
    true
  )
})

test("does not select sandbox chrome for other imported Wikidot source pages", () => {
  assert.equal(
    shouldUseSandboxWikidotChrome({
      site: { slug: "scp-wiki", from_wikidot: true },
      page: { from_wikidot: true },
      page_revision: { from_wikidot: true },
      wikidot_snapshot: { source_site: "scp-wiki" }
    }),
    false
  )
})

test("uses source sandbox title and suppresses the local-only tagline", () => {
  const data = {
    site: { name: "Sandbox for Codex", tagline: "Local Wikidot behavior sandbox" },
    wikidot_snapshot: { source_site: "sandbox-for-codex" }
  }

  assert.equal(resolveWikidotSiteTitle(data), "Sandbox For Codex")
  assert.equal(resolveWikidotSiteTagline(data), null)
})

test("resolves the authenticated user name for Wikidot-compatible chrome", () => {
  assert.equal(
    resolveWikidotSessionUserName({
      user_session: { user: { name: "scpaiueouiuiuiui", slug: "account-slug" } }
    }),
    "scpaiueouiuiuiui"
  )
  assert.equal(
    resolveWikidotSessionUserName({
      user_session: { user: { name: "", slug: "account-slug" } }
    }),
    "account-slug"
  )
})
