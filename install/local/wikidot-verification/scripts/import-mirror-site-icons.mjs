#!/usr/bin/env node

// Materialize a mirrored Wikidot site's browser icons into Wikijump.
//
// The source site's own document head says which icons it publishes
// (favicon, iOS touch icons, Windows tile). Each declared icon is fetched
// through the identity-bound public evidence response cache, so a retained
// response is replayed and a miss is acquired once under the cache's
// acquisition barrier. The bytes and their observed media type are stored
// with Deepwell's `site_icon_set`; a kind the source no longer declares is
// cleared, so a re-run refreshes and removes stale imported icons.

import crypto from "node:crypto"
import fs from "node:fs/promises"
import path from "node:path"

import { loadPlaywright } from "../src/browser-session.mjs"
import {
  createBrowserRequestGate,
  createBrowserResponseCache,
  defaultPublicEvidenceResponseCacheOptions,
  installBrowserRequestGate
} from "../src/browser-request-gate.mjs"
import { deepwellRpcAuthorization } from "../src/deepwell-rpc-auth.mjs"
import { runCliIfMain } from "../src/cli-entry.mjs"

export const ICON_KINDS = Object.freeze(["favicon", "ios", "windows"])
const ICON_ROUTES = Object.freeze({
  favicon: "/local--favicon/",
  ios: "/local--iosicon/",
  windows: "/local--wp8icon/"
})
const MIMES = new Set(["image/gif", "image/png", "image/x-icon", "image/jpeg"])

function attribute(tag, name) {
  const match = new RegExp(`\\s${name}\\s*=\\s*("([^"]*)"|'([^']*)'|([^\\s>]+))`, "iu").exec(tag)
  return match ? (match[2] ?? match[3] ?? match[4]) : null
}

/** The icon each kind's route serves, as declared in the source head.
 * iOS declares several sizes of one uploaded image; the unsized-name
 * `iosicon.png` (or the largest declared size) is the stored original. */
export function declaredSiteIcons(html, origin) {
  const head = html.slice(0, Math.max(0, html.search(/<\/head\s*>/iu)) || html.length)
  const icons = { favicon: [], ios: [], windows: [] }
  for (const tag of head.match(/<(?:link|meta)\b[^>]*>/giu) ?? []) {
    const href = attribute(tag, "href") ?? (/msapplication-TileImage/iu.test(tag) ? attribute(tag, "content") : null)
    if (!href) continue
    const url = new URL(href, origin)
    if (url.origin !== new URL(origin).origin) continue
    for (const kind of ICON_KINDS) {
      if (url.pathname.startsWith(ICON_ROUTES[kind])) {
        const sizes = attribute(tag, "sizes")
        const size = sizes ? Number.parseInt(sizes, 10) : 0
        icons[kind].push({ url: url.href, size: Number.isFinite(size) ? size : 0 })
      }
    }
  }
  const chosen = {}
  for (const kind of ICON_KINDS) {
    const candidates = [...new Map(icons[kind].map((icon) => [icon.url, icon])).values()]
    if (candidates.length === 0) continue
    candidates.sort((left, right) => right.size - left.size || left.url.localeCompare(right.url))
    chosen[kind] = candidates[0].url
  }
  return chosen
}

export function iconMime(contentType) {
  const mime = (contentType ?? "").split(";", 1)[0].trim().toLowerCase()
  if (!MIMES.has(mime)) throw new Error(`unsupported site icon media type: ${contentType}`)
  return mime
}

function parseArgs(argv) {
  const args = {}
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index]
    const value = argv[index + 1]
    if (!["--site", "--source-origin", "--rpc-url", "--session-file", "--cache-dir", "--cache-identity", "--browser-root", "--receipt"].includes(flag) || !value) {
      throw new Error(`unknown or incomplete option: ${flag}`)
    }
    args[flag.slice(2).replace(/-([a-z])/gu, (_, c) => c.toUpperCase())] = value
  }
  for (const name of ["site", "sourceOrigin", "rpcUrl", "sessionFile", "receipt"]) {
    if (!args[name]) throw new Error(`--${name.replace(/[A-Z]/gu, (c) => `-${c.toLowerCase()}`)} is required`)
  }
  if ((args.cacheDir === undefined) !== (args.cacheIdentity === undefined)) {
    throw new Error("--cache-dir and --cache-identity go together")
  }
  return args
}

async function rpc(args, session, method, params) {
  const response = await fetch(args.rpcUrl, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      authorization: deepwellRpcAuthorization(),
      "x-deepwell-session-token": session
    },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params })
  })
  const body = await response.json()
  if (body.error) throw new Error(`${method} failed: ${body.error.message}`)
  return body.result
}

export async function importMirrorSiteIcons(args) {
  const session = JSON.parse(await fs.readFile(args.sessionFile, "utf8")).session_token
  if (!session) throw new Error("session file has no session_token")
  const gate = createBrowserRequestGate({ intervalMs: 0 })
  const responseCache = createBrowserResponseCache({
    ...defaultPublicEvidenceResponseCacheOptions(),
    ...(args.cacheDir ? { persistentDir: args.cacheDir, persistentIdentity: args.cacheIdentity } : {}),
    cacheDocuments: true,
    evidenceReplay: true
  })
  await responseCache.load()
  const { chromium } = loadPlaywright(args.browserRoot)
  const browser = await chromium.launch()
  const fetched = {}
  let declared
  try {
    const context = await browser.newContext({ serviceWorkers: "block" })
    await installBrowserRequestGate(context, { gate, responseCache })
    const page = await context.newPage()
    // Only the documents themselves are needed; the home page's own
    // subresources are not requested.
    await page.route("**/*", (route) =>
      route.request().resourceType() === "document" ? route.fallback() : route.abort("blockedbyclient")
    )
    const home = await page.goto(new URL("/", args.sourceOrigin).href, { waitUntil: "commit" })
    if (home?.status() !== 200) throw new Error(`source home answered ${home?.status()}`)
    declared = declaredSiteIcons(await home.text(), args.sourceOrigin)
    for (const [kind, url] of Object.entries(declared)) {
      const response = await page.goto(url, { waitUntil: "commit" })
      if (response?.status() !== 200) throw new Error(`${url} answered ${response?.status()}`)
      const bytes = await response.body()
      fetched[kind] = { url, mime: iconMime(response.headers()["content-type"]), bytes }
    }
    await context.close()
  } finally {
    await browser.close().catch(() => {})
    await gate.flush()
  }

  const results = []
  for (const kind of ICON_KINDS) {
    const icon = fetched[kind]
    const params = icon
      ? { site: args.site, kind, ip_address: "127.0.0.1", source: icon.url, mime: icon.mime, data: [...icon.bytes] }
      : { site: args.site, kind, ip_address: "127.0.0.1", source: null }
    await rpc(args, session, "site_icon_set", params)
    results.push(icon
      ? { kind, source: icon.url, mime: icon.mime, bytes: icon.bytes.length, sha256: crypto.createHash("sha256").update(icon.bytes).digest("hex") }
      : { kind, source: null, cleared: true })
  }
  const receipt = {
    schema: "wikijump.mirror_site_icons_receipt.v1",
    completed_at: new Date().toISOString(),
    site: args.site,
    source_origin: args.sourceOrigin,
    cache: args.cacheDir ? { dir: path.resolve(args.cacheDir), identity: args.cacheIdentity } : "default public evidence cache",
    icons: results
  }
  await fs.writeFile(args.receipt, `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" })
  return receipt
}

export async function main(argv) {
  const receipt = await importMirrorSiteIcons(parseArgs(argv))
  console.log(JSON.stringify(receipt.icons))
  return 0
}

await runCliIfMain(import.meta.url, main)
