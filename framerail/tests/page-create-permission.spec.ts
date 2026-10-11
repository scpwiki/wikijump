import {
  expect,
  installNativeEventListenerProbe,
  test,
  waitForNativeEventListener,
  waitForSvelteDelegatedHandler
} from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token"
}
const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`

// Wikidot's create-denial sentence, shown in the permission dialog.
const CREATE_DENIED_MESSAGE =
  "Sorry, you can not create a new page in this category. Only members of this site, site administrators and perhaps selected moderators are allowed to do it."

// Denied slugs are never created, so each test can reuse the same missing slug.
const DENIED_SLUG = "create-permission-denied-probe"

const pageCreateWritesFor = async (
  request: import("./hermetic-playwright").APIRequestContext,
  slug: string
) => {
  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  return (writes.pageCreate as { params: { slug?: string } }[]).filter(
    (entry) => entry.params.slug === slug
  )
}

test("anonymous Create page on a missing page shows the denial without the editor", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const missingResponse = await page.goto(`/${DENIED_SLUG}`)
  expect(missingResponse?.status()).toBe(404)
  await expect(page.locator("#create-it-now-link")).toHaveText("Create page")

  await waitForSvelteDelegatedHandler(page, "#create-it-now-link a")
  await page.locator("#create-it-now-link a").click()

  await expect(page.getByText(CREATE_DENIED_MESSAGE)).toBeVisible()
  await expect(page).toHaveURL(new RegExp(`/${DENIED_SLUG}$`, "u"))
  await expect(page.locator("textarea.editor-wikitext")).toHaveCount(0)
  await expect(page.locator("input[name='title']")).toHaveCount(0)
  await expect(page.locator(".page-create-header")).toHaveCount(0)
  expect(await pageCreateWritesFor(request, DENIED_SLUG)).toHaveLength(0)
})

test("anonymous direct /edit/true on a missing page renders no editor", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`/${DENIED_SLUG}/edit/true`)
  expect(response?.status()).toBe(404)

  await expect(page.getByText(CREATE_DENIED_MESSAGE)).toBeVisible()
  await expect(page.locator("textarea.editor-wikitext")).toHaveCount(0)
  await expect(page.locator("input[name='title']")).toHaveCount(0)
  await expect(page.locator(".page-create-header")).toHaveCount(0)
  await expect(page.locator("#create-it-now-link")).toHaveCount(1)
  expect(await pageCreateWritesFor(request, DENIED_SLUG)).toHaveLength(0)
})

test("anonymous direct /edit on a missing page renders no editor", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`/${DENIED_SLUG}/edit`)
  expect(response?.status()).toBe(404)

  await expect(page.getByText(CREATE_DENIED_MESSAGE)).toBeVisible()
  await expect(page.locator("textarea.editor-wikitext")).toHaveCount(0)
  await expect(page.locator("input[name='title']")).toHaveCount(0)
  expect(await pageCreateWritesFor(request, DENIED_SLUG)).toHaveLength(0)
})

test("anonymous forged create POST on a missing page writes nothing", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.request.post(`/${DENIED_SLUG}/edit/true?/edit`, {
    headers: SITE_HEADERS,
    form: {
      siteId: "6000005",
      pageId: "0",
      lastRevisionId: "0",
      title: "Forged",
      altTitle: "",
      wikitext: "forged source",
      tags: "",
      comments: "forged"
    }
  })
  expect(response.ok()).toBe(true)
  expect(await response.text()).not.toContain('"type":"success"')
  expect(await pageCreateWritesFor(request, DENIED_SLUG)).toHaveLength(0)
})

test("authorized direct /edit/true on a missing page opens the editor and creates it", async ({
  page,
  request
}) => {
  const slug = "create-permission-authorized-probe"
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const response = await page.goto(`/${slug}/edit/true`)
  expect(response?.status()).toBe(404)

  await expect(page.locator("textarea[name='wikitext']")).toBeVisible()
  await waitForNativeEventListener(page, "#editor", "submit")
  await expect(page.getByText(CREATE_DENIED_MESSAGE)).toHaveCount(0)

  await page.locator("input[name='title']").fill("Authorized create probe")
  await page.locator("textarea[name='wikitext']").fill("Authorized body")
  await page.getByRole("button", { name: "save", exact: true }).click()

  await expect.poll(async () => (await pageCreateWritesFor(request, slug)).length).toBe(1)
})
