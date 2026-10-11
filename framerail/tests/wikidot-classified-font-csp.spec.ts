import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

// The audited Renner origin. `component:classified-decoration-base` imports
// its stylesheet and WOFF2 from here in both retained shared-corpus copies,
// so the browser has to be allowed to fetch CSS and font bytes from it
// (issue #2213).
const AUDITED_FONT_ORIGIN = "https://indestructibletype-fonthosting.github.io"

const cspDirectiveSources = (policy: string, directive: string): string[] => {
  const segment = policy
    .split(";")
    .map((part) => part.trim())
    .find((part) => part.startsWith(`${directive} `) || part === directive)
  return segment ? segment.split(/\s+/u).slice(1) : []
}

test("the served CSP allowlists the audited classified-decoration font origin", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`${APP_URL}/theme:yossistyle`)
  expect(response?.status()).toBe(200)

  const policy = response?.headers()["content-security-policy"] ?? ""
  expect(policy).toContain("default-src")
  expect(cspDirectiveSources(policy, "style-src"), `style-src of ${policy}`).toContain(
    AUDITED_FONT_ORIGIN
  )
  expect(cspDirectiveSources(policy, "font-src"), `font-src of ${policy}`).toContain(
    AUDITED_FONT_ORIGIN
  )

  // The audited origin serves CSS and font bytes only. It must never appear
  // in a directive that would make it an executable or framing dependency,
  // and no directive may fall back to a scheme-wide external source.
  for (const directive of ["script-src", "connect-src", "frame-src", "worker-src"]) {
    expect(
      cspDirectiveSources(policy, directive),
      `${directive} of ${policy}`
    ).not.toContain(AUDITED_FONT_ORIGIN)
  }
  for (const directive of ["style-src", "font-src"]) {
    expect(cspDirectiveSources(policy, directive)).not.toContain("https:")
  }
  expect(cspDirectiveSources(policy, "script-src")).not.toContain("https:")
})
