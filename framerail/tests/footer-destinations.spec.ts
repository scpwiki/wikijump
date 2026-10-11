import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

/**
 * Footer destinations must lead where their label says, on real served
 * pages, in both layouts, and must never resolve against the wiki being
 * viewed.
 *
 * Native layout: Docs, Terms, Privacy and Security are platform-global
 * items. Their destinations are instance-level absolute URLs, so a footer
 * click can never be captured by the current site's origin and never
 * collapses onto the current homepage.
 *
 * Wikidot-compatible layout: Help, Terms of Service, Privacy, Report a bug
 * and Flag as objectionable reproduce the current live Wikidot footer,
 * whose first three items point at `www.wikidot.com` and whose report
 * controls are inert anchors. A footer item that resolves to plain `/` is
 * the placeholder defect this spec guards.
 */

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const PLATFORM_HEADERS = {
  "X-Wikijump-Site-Id": "6000006",
  "X-Wikijump-Site-Slug": "www"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

type NativeFooterKey = "terms-conditions" | "privacy" | "docs" | "security"

const NATIVE_FOOTER_KEYS: NativeFooterKey[] = [
  "terms-conditions",
  "privacy",
  "docs",
  "security"
]
const NATIVE_FOOTER_ENVIRONMENT_VARIABLES: Record<NativeFooterKey, string> = {
  "terms-conditions": "PUBLIC_WIKIJUMP_FOOTER_TERMS_CONDITIONS_URL",
  privacy: "PUBLIC_WIKIJUMP_FOOTER_PRIVACY_URL",
  docs: "PUBLIC_WIKIJUMP_FOOTER_DOCS_URL",
  security: "PUBLIC_WIKIJUMP_FOOTER_SECURITY_URL"
}
const DOCUMENTED_NATIVE_FOOTER_HREFS: Record<NativeFooterKey, string> = {
  "terms-conditions": "https://wikijump.com/legal/terms-of-service",
  privacy: "https://wikijump.com/legal/privacy-policy",
  docs: "https://wikijump.com/docs",
  security: "https://wikijump.com/legal/security"
}

/**
 * The destination this instance serves: configured override, else
 * documented.
 */
const expectedNativeHref = (key: NativeFooterKey) =>
  process.env[NATIVE_FOOTER_ENVIRONMENT_VARIABLES[key]]?.trim() ||
  DOCUMENTED_NATIVE_FOOTER_HREFS[key]

type NativeFooterSnapshot = {
  count: number
  hrefs: Array<string | null>
  rels: Array<string | null>
  texts: string[]
  origin: string
  host: string
}

const collectNativeFooter = (
  page: import("@playwright/test").Page
): Promise<NativeFooterSnapshot> =>
  page.evaluate(() => {
    const links = Array.from(
      document.querySelectorAll<HTMLAnchorElement>(".footer-items .footer-item a")
    )
    return {
      count: links.length,
      hrefs: links.map((link) => link.getAttribute("href")),
      rels: links.map((link) => link.getAttribute("rel")),
      texts: links.map((link) => link.textContent ?? ""),
      origin: window.location.origin,
      host: window.location.host
    }
  })

const assertNativeFooterDestinations = async (
  page: import("@playwright/test").Page,
  route: string
) => {
  const footer = await collectNativeFooter(page)
  expect(footer.count, `${route} footer item count`).toBe(NATIVE_FOOTER_KEYS.length)

  for (const [index, key] of NATIVE_FOOTER_KEYS.entries()) {
    const href = footer.hrefs[index]
    const expected = expectedNativeHref(key)
    expect(href, `${route} ${key} served destination`).toBe(expected)
    expect(href, `${route} ${key} must not be empty`).toMatch(/\S/u)
    expect(href, `${route} ${key} must not be the bare root`).not.toBe("/")
    expect(href, `${route} ${key} must not be a same-site path`).not.toMatch(/^\/[^/]/u)
    expect(href, `${route} ${key} must not inherit a wiki origin`).not.toBe(footer.origin)
    expect(new URL(String(href)).host, `${route} ${key} host`).not.toBe(footer.host)
    // External platform destinations use normal safe link behaviour.
    expect(footer.rels[index], `${route} ${key} rel`).toBe("external")
    expect(footer.texts[index], `${route} ${key} keeps its label`).not.toBe("")
    // The advertised action never leads back to the viewed wiki.
    const resolved = new URL(String(href), footer.origin)
    expect(
      resolved.origin === footer.origin && resolved.pathname === "/",
      `${route} ${key} must not resolve to the wiki root`
    ).toBe(false)
  }
}

const NATIVE_ROUTES = [
  { path: "/-/login", headers: SITE_HEADERS, label: "native account page" },
  { path: "/-/user/guest", headers: SITE_HEADERS, label: "native profile page" },
  { path: "/platform:search", headers: PLATFORM_HEADERS, label: "platform site" }
]

test("native footer items lead to their platform destinations, not the wiki", async ({
  page
}) => {
  test.setTimeout(240_000)

  for (const { path, headers, label } of NATIVE_ROUTES) {
    await page.setExtraHTTPHeaders(headers)
    const response = await page.goto(`${APP_URL}${path}`)
    expect(response?.status(), `${label} status`).toBe(200)
    await expect(page.locator(".footer-inner")).toBeVisible()
    await assertNativeFooterDestinations(page, label)
  }
})

test("native footer destinations do not depend on the viewed wiki", async ({ page }) => {
  test.setTimeout(240_000)

  const seen: string[] = []
  for (const { path, headers } of NATIVE_ROUTES) {
    await page.setExtraHTTPHeaders(headers)
    const response = await page.goto(`${APP_URL}${path}`)
    expect(response?.status(), `${path} status`).toBe(200)
    seen.push((await collectNativeFooter(page)).hrefs.join(" | "))
  }

  // Every served site, wiki or platform, exposes the same instance-level
  // platform destinations.
  expect(new Set(seen).size).toBe(1)
})

test("native footer destinations follow instance configuration", async ({ page }) => {
  test.setTimeout(240_000)

  // Run this spec with `PUBLIC_WIKIJUMP_FOOTER_*_URL` set to prove a
  // self-hosted instance is not forced onto the public hostname.
  const configured = Object.fromEntries(
    NATIVE_FOOTER_KEYS.map((key) => [
      key,
      process.env[NATIVE_FOOTER_ENVIRONMENT_VARIABLES[key]]?.trim() || undefined
    ])
  )
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`${APP_URL}/-/login`)
  expect(response?.status()).toBe(200)

  const hrefs = (await collectNativeFooter(page)).hrefs
  for (const [index, key] of NATIVE_FOOTER_KEYS.entries()) {
    if (configured[key]) {
      // A configured instance serves exactly what the operator configured.
      expect(hrefs[index], `${key} configured destination`).toBe(configured[key])
    } else {
      expect(hrefs[index], `${key} documented destination`).toBe(
        DOCUMENTED_NATIVE_FOOTER_HREFS[key]
      )
    }
  }

  if (Object.values(configured).some(Boolean)) {
    // Not one configured item falls back to the public platform hostname.
    for (const [index, key] of NATIVE_FOOTER_KEYS.entries()) {
      if (configured[key]) {
        expect(String(hrefs[index])).not.toBe(DOCUMENTED_NATIVE_FOOTER_HREFS[key])
      }
    }
  }
})

test("imported Wikidot-layout footer reproduces the live Wikidot destinations", async ({
  page
}) => {
  test.setTimeout(120_000)

  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`${APP_URL}/wikidot-login-probe`)
  expect(response?.status()).toBe(200)
  await waitForSvelteDelegatedHandler(page, ".login-status-sign-in", "click")

  const footer = page.locator(".options")
  await expect(footer).toBeVisible()
  const links = footer.locator("a")
  await expect(links).toHaveCount(5)

  // Current live Wikidot footer: Help, Terms of Service and Privacy are
  // absolute www.wikidot.com URLs; Report a bug and Flag as objectionable are
  // inert anchors whose listeners open the in-page report modules.
  const expected = [
    "https://www.wikidot.com/doc",
    "https://www.wikidot.com/legal:terms-of-service",
    "https://www.wikidot.com/legal:privacy-policy",
    "javascript:;",
    "javascript:;"
  ]
  const hrefs = await links.evaluateAll((anchors) =>
    anchors.map((anchor) => anchor.getAttribute("href"))
  )
  expect(hrefs).toEqual(expected)

  const rootResolving = await links.evaluateAll((anchors) =>
    anchors
      .map((anchor) => ({
        href: anchor.getAttribute("href"),
        origin: window.location.origin
      }))
      .filter(({ href, origin }) => {
        if (href === null || href === "javascript:;") return false
        const resolved = new URL(href, origin)
        return resolved.origin === origin && resolved.pathname === "/"
      })
      .map(({ href }) => href)
  )
  expect(rootResolving).toEqual([])
})

test("no footer item resolves to the current wiki root", async ({ page }) => {
  test.setTimeout(240_000)

  const routes = [
    ...NATIVE_ROUTES.map(({ path, headers }) => ({
      path,
      headers,
      selector: ".footer-items .footer-item a"
    })),
    {
      path: "/wikidot-login-probe",
      headers: SITE_HEADERS,
      selector: ".options a"
    }
  ]

  for (const { path, headers, selector } of routes) {
    await page.setExtraHTTPHeaders(headers)
    const response = await page.goto(`${APP_URL}${path}`)
    expect(response?.status(), `${path} status`).toBe(200)

    const rootResolving = await page.locator(selector).evaluateAll((anchors) =>
      anchors
        .map((anchor) => ({
          href: anchor.getAttribute("href"),
          origin: window.location.origin
        }))
        .filter(({ href, origin }) => {
          if (href === null) return true
          if (href === "javascript:;") return false
          const resolved = new URL(href, origin)
          return resolved.origin === origin && resolved.pathname === "/"
        })
        .map(({ href }) => href)
    )
    expect(rootResolving, `${path} items resolving to the wiki root`).toEqual([])
  }
})
