import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("the email action restores hidden Wikidot email spans in the article DOM", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/wikidot-tabview", { waitUntil: "load" })

  await page.locator("#page-content").evaluate((content) => {
    for (const [index, encoded] of [
      "vog.isfn|ofni#vog.isfn|ofni",
      "moc.elpmaxe|troppus#em liame"
    ].entries()) {
      const span = document.createElement("span")
      span.className = "wiki-email"
      span.dataset.testid = `email-${index}`
      span.textContent = encoded
      content.append(span)
    }

    const apostrophe = document.createElement("span")
    apostrophe.className = "wiki-email"
    apostrophe.dataset.testid = "email-apostrophe"
    apostrophe.innerHTML = "moc.elpmaxe|arah&#39;o#moc.elpmaxe|arah&#39;o"
    content.append(apostrophe, document.createTextNode("."))

    const malformed = document.createElement("span")
    malformed.className = "wiki-email"
    malformed.dataset.testid = "email-malformed"
    malformed.textContent = "not-an-encoded-address"
    content.append(malformed)
  })

  await page.locator("#page-content").evaluate(async (content) => {
    const module = (await import(
      /* @vite-ignore */
      new URL("/src/lib/wikidot/wikidot-email-obfuscation.js", location.href).href
    )) as typeof import("../src/lib/wikidot/wikidot-email-obfuscation.js")
    module.wikidotEmailObfuscation(content)
  })

  const plainEmail = page.locator('#page-content [data-testid="email-0"]')
  const labeledEmail = page.locator('#page-content [data-testid="email-1"]')
  const malformedEmail = page.locator('#page-content [data-testid="email-malformed"]')
  const apostropheEmail = page.locator('#page-content [data-testid="email-apostrophe"]')
  await expect(plainEmail).toBeVisible()
  await expect(plainEmail.getByRole("link")).toHaveAttribute(
    "href",
    "mailto:info@nfsi.gov"
  )
  await expect(plainEmail).toHaveText("info@nfsi.gov")
  await expect(labeledEmail.getByRole("link")).toHaveAttribute(
    "href",
    "mailto:support@example.com"
  )
  await expect(labeledEmail).toHaveText("email me")
  await expect(apostropheEmail).toBeVisible()
  await expect(apostropheEmail.getByRole("link")).toHaveAttribute(
    "href",
    "mailto:o'hara@example.com"
  )
  await expect(apostropheEmail).toHaveText("o'hara@example.com")
  await expect(page.locator("#page-content")).toContainText("o'hara@example.com.")
  await expect(malformedEmail).toBeHidden()
  await expect(malformedEmail.locator("a")).toHaveCount(0)
})
