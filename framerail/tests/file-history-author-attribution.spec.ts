import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

/**
 * Encode a value in SvelteKit's flat devalue shape, as `deserialize`
 * expects.
 */
const devalue = (value: unknown): string => {
  const flat: unknown[] = []
  const add = (item: unknown): number => {
    const index = flat.length
    flat.push(null)
    if (Array.isArray(item)) {
      flat[index] = item.map(add)
    } else if (item !== null && typeof item === "object") {
      const encoded: Record<string, number> = {}
      for (const [key, child] of Object.entries(item)) encoded[key] = add(child)
      flat[index] = encoded
    } else {
      flat[index] = item
    }
    return index
  }
  add(value)
  return JSON.stringify(flat)
}

const successBody = (value: unknown) =>
  JSON.stringify({ type: "success", status: 200, data: devalue(value) })

const file = {
  file_id: 77,
  file_created_at: "2026-10-01T00:00:00Z",
  file_updated_at: null,
  file_deleted_at: null,
  page_id: 3000340,
  revision_id: 88,
  revision_type: "create",
  revision_created_at: "2026-10-01T00:00:00Z",
  revision_number: 0,
  revision_user_id: 456,
  name: "attribution-probe.gif",
  data: null,
  mime: "image/gif",
  size: 1748073,
  s3_hash: "fixture-hash",
  revision_comments: "",
  hidden_fields: []
}

const revision = (
  revisionId: number,
  revisionNumber: number,
  userId: number,
  author: unknown
) => ({
  revision_id: revisionId,
  revision_type: "create",
  created_at: "2026-10-01T00:00:00Z",
  revision_number: revisionNumber,
  file_id: 77,
  page_id: 3000340,
  site_id: 6000005,
  user_id: userId,
  author,
  name: "attribution-probe.gif",
  s3_hash: null,
  mime: "image/gif",
  size: 1748073,
  changes: [],
  comments: null,
  hidden: []
})

const openHistory = async (page: import("@playwright/test").Page) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [file] })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/fileHistory",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({
          res: [
            revision(89, 1, 456, {
              "user-id": 456,
              "user-slug": "seek-gull",
              "user-name": "SeekGull",
              "user-karma": 3,
              "user-avatar-data": "",
              "user-profile-url": "/-/user/seek-gull"
            }),
            revision(90, 0, -1, null)
          ]
        })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await page
    .locator(".file-row")
    .getByRole("link", { name: "history", exact: true })
    .click()
  await expect(page.locator(".revision-row")).toHaveCount(2)
}

test("File History shows the resolved uploader name instead of a raw user ID", async ({
  page
}) => {
  await openHistory(page)

  // Newest first: the attributed revision renders the public identity link.
  const attributedUser = page.locator(".revision-row .revision-attribute.user").first()
  await expect(attributedUser.getByRole("link")).toHaveText("SeekGull")
  await expect(attributedUser.getByRole("link")).toHaveAttribute(
    "href",
    "/-/user/seek-gull"
  )
  await expect(attributedUser).not.toContainText("456")
})

test("File History without a public identity keeps the deleted-user marker", async ({
  page
}) => {
  await openHistory(page)

  // The synthetic import actor (-1) has no public identity: the raw ID must
  // not appear as row text, and the pane keeps the page History pane's
  // deleted-user marker with the stable ID only as audit data.
  const unattributedUser = page.locator(".revision-row .revision-attribute.user").last()
  await expect(unattributedUser).not.toContainText("-1")
  const marker = unattributedUser.locator(".printuser.deleted")
  await expect(marker).toHaveCount(1)
  await expect(marker).toHaveAttribute("data-id", "-1")
})
