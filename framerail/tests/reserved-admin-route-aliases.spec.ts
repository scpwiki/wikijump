import { expect, test } from "./hermetic-playwright"

const SITE_CONTEXTS = [
  {
    name: "imported mirror",
    headers: {
      "X-Wikijump-Site-Id": "6000005",
      "X-Wikijump-Site-Slug": "scp-wiki"
    }
  },
  {
    name: "default template",
    headers: {
      "X-Wikijump-Site-Id": "6000006",
      "X-Wikijump-Site-Slug": "template-en"
    }
  }
]
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
test("anonymous canonical and alternate admin routes share the denied response", async ({
  page
}) => {
  for (const site of SITE_CONTEXTS) {
    await page.setExtraHTTPHeaders(site.headers)

    const canonical = await page.goto(`${APP_URL}/_admin`)
    expect(canonical?.status(), site.name).toBe(401)
    const canonicalBody = await page.locator("body").innerText()
    const canonicalTitle = await page.title()
    const canonicalHtml = await page.content()
    expect(canonicalBody).not.toContain("Admin Panel")
    expect(canonicalBody).not.toContain("[[module Redirect")
    expect(canonicalTitle).not.toContain("Admin Panel")
    expect(canonicalHtml).not.toContain("Admin Panel")
    expect(canonicalHtml).not.toContain('[[module Redirect destination="/-/admin"]]')

    for (const path of [
      "/_admin/norender/true",
      "/_admin?norender=true",
      "/printer--friendly//_admin"
    ]) {
      const response = await page.goto(`${APP_URL}${path}`)
      expect(response?.status(), `${site.name} ${path}`).toBe(canonical?.status())
      expect(new URL(page.url()).pathname, path).toBe("/_admin")

      const body = await page.locator("body").innerText()
      expect(body, `${site.name} ${path}`).toBe(canonicalBody)
      expect(await page.title(), `${site.name} ${path}`).toBe(canonicalTitle)
      const html = await page.content()
      expect(html, `${site.name} ${path}`).not.toContain("Admin Panel")
      expect(html, `${site.name} ${path}`).not.toContain(
        '[[module Redirect destination="/-/admin"]]'
      )
      expect(body, path).not.toContain("Admin Panel")
      expect(body, path).not.toContain("[[module Redirect")
    }
  }

  const fixturePort = process.env.PLAYWRIGHT_FIXTURE_PORT
  expect(fixturePort).toBeTruthy()
  const articleReads = await page.request.get(
    `http://127.0.0.1:${fixturePort}/last-article-read-requests`
  )
  expect(articleReads.ok()).toBe(true)
  expect((await articleReads.json()).articleView).toEqual([])
})

test("authorized admin retains the canonical route after both aliases", async ({
  page
}) => {
  const appPort = process.env.PLAYWRIGHT_APP_PORT
  expect(appPort).toBeTruthy()
  await page.context().addCookies([
    {
      name: "wikijump_token",
      value: "fixture-session-token",
      url: `${APP_URL}/`
    }
  ])

  for (const site of SITE_CONTEXTS) {
    await page.setExtraHTTPHeaders(site.headers)
    for (const path of ["/_admin/norender/true", "/printer--friendly//_admin"]) {
      const response = await page.goto(`${APP_URL}${path}`)
      expect(response?.status(), `${site.name} ${path}`).toBe(200)
      expect(new URL(page.url()).pathname, path).toBe("/_admin")
      await expect(page.locator("#sm-general-name"), path).toBeVisible()
    }
  }
})

test("authenticated non-admin member remains denied through both aliases", async ({
  page
}) => {
  const appPort = process.env.PLAYWRIGHT_APP_PORT
  expect(appPort).toBeTruthy()
  await page.context().addCookies([
    {
      name: "wikijump_token",
      value: "fixture-authenticated-session-token",
      url: `${APP_URL}/`
    }
  ])

  for (const site of SITE_CONTEXTS) {
    await page.setExtraHTTPHeaders(site.headers)
    const canonical = await page.goto(`${APP_URL}/_admin`)
    expect(canonical?.status(), site.name).toBe(401)
    const canonicalBody = await page.locator("body").innerText()
    const canonicalTitle = await page.title()
    const canonicalHtml = await page.content()
    expect(canonicalBody).not.toContain("Admin Panel")
    expect(canonicalTitle).not.toContain("Admin Panel")
    expect(canonicalHtml).not.toContain("Admin Panel")
    expect(canonicalHtml).not.toContain('[[module Redirect destination="/-/admin"]]')

    for (const path of ["/_admin/norender/true", "/printer--friendly//_admin"]) {
      const response = await page.goto(`${APP_URL}${path}`)
      expect(response?.status(), `${site.name} ${path}`).toBe(canonical?.status())
      expect(new URL(page.url()).pathname, path).toBe("/_admin")
      expect(await page.locator("body").innerText(), path).toBe(canonicalBody)
      expect(await page.title(), `${site.name} ${path}`).toBe(canonicalTitle)
      const html = await page.content()
      expect(html, `${site.name} ${path}`).not.toContain("Admin Panel")
      expect(html, `${site.name} ${path}`).not.toContain(
        '[[module Redirect destination="/-/admin"]]'
      )
    }
  }
})
