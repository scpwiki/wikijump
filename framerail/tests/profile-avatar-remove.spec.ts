import type { Route } from "@playwright/test"

import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AVATAR_OWNER_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-authenticated-session-token",
  origin: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
}
const REMOVE_LABEL = "Remove profile image"

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

/**
 * Decode the superforms JSON payload of a multipart action request into
 * its submitted fields. Superforms stores values in a flat array and
 * references them by index; `-1` marks an undefined field.
 */
const submittedFields = (body: string): Record<string, unknown> => {
  const match = /name="__superform_json"\r\n\r\n([\s\S]*?)\r\n--/u.exec(body)
  if (!match) throw new Error("action request carried no superform payload")
  const flat = JSON.parse(match[1]) as unknown[]
  const root = flat[0] as Record<string, number>
  return Object.fromEntries(
    Object.entries(root).map(([key, index]) => [
      key,
      index === -1 ? undefined : flat[index]
    ])
  )
}

const successBody = (value: unknown) =>
  JSON.stringify({ type: "success", status: 200, data: devalue(value) })

/**
 * Open the own-account profile editor. The profile is the fixture owner's,
 * who has a stored avatar; the action endpoint is intercepted so nothing
 * reaches Deepwell, and every submitted action is recorded for
 * assertions.
 */
const openOwnEditor = async (page: import("@playwright/test").Page, actions: Route[]) => {
  await page.setExtraHTTPHeaders(AVATAR_OWNER_HEADERS)
  await page.route(
    (url) => url.search === "?/userEdit",
    async (route) => {
      actions.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({})
      })
    }
  )
  await page.goto("/-/user", { waitUntil: "domcontentloaded" })
  await expect(page.locator(".user-attribute.name")).toHaveText("Avatar Probe")
  await expect(page.locator(".user-attribute.avatar img")).toBeVisible()
  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(page.locator("#editor")).toBeVisible()
}

test("the own profile with a stored avatar offers a labeled Remove action", async ({
  page
}) => {
  const actions: Route[] = []
  await openOwnEditor(page, actions)

  const remove = page.getByRole("button", { name: REMOVE_LABEL, exact: true })
  await expect(remove).toBeVisible()
  await expect(remove).toHaveAttribute("type", "button")
  await expect(remove).toHaveClass(/button-remove-avatar/u)
  expect(actions).toHaveLength(0)
})

test("Remove is a separate control from the file upload and from Save", async ({
  page
}) => {
  const actions: Route[] = []
  await openOwnEditor(page, actions)

  const remove = page.getByRole("button", { name: REMOVE_LABEL, exact: true })
  const upload = page.locator("input#avatar[type=file]")
  await expect(upload).toHaveCount(1)
  await expect(upload).toHaveAttribute("accept", "image/png,image/jpeg,image/bmp")
  await expect(upload).toHaveAttribute("name", "avatar")
  await expect(remove).toHaveCount(1)
  expect(await remove.evaluate((element) => element.tagName)).toBe("BUTTON")

  // The removal button is not the form's submit control, so activating it
  // only changes the draft; Save remains the single submitting control.
  await expect(page.locator("#editor button[type=submit]")).toHaveText(/./u)
  await expect(page.locator("#editor button[type=submit]")).toHaveCount(1)
  expect(actions).toHaveLength(0)
})

test("cancelling a pending removal sends nothing and restores the stored avatar", async ({
  page
}) => {
  const actions: Route[] = []
  page.on("dialog", (dialog) => void dialog.dismiss())
  await openOwnEditor(page, actions)

  await page.getByRole("button", { name: REMOVE_LABEL, exact: true }).click()
  await expect(page.getByRole("status")).toBeVisible()
  await expect(page.locator("#editor .button-keep-avatar")).toBeVisible()

  await page.locator(".button-cancel").click()
  await expect(page.locator("#editor")).toHaveCount(0)
  expect(actions).toHaveLength(0)

  // Reopening starts from the accepted state: the image is still stored and
  // no removal is pending.
  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(page.locator("#editor")).toBeVisible()
  await expect(page.getByRole("status")).toHaveCount(0)
  await expect(
    page.getByRole("button", { name: REMOVE_LABEL, exact: true })
  ).toBeVisible()
  expect(actions).toHaveLength(0)
})

test("a plain Save with no avatar change never carries a removal", async ({ page }) => {
  const actions: Route[] = []
  await openOwnEditor(page, actions)

  await page.locator("#editor button[type=submit]").click()
  await expect.poll(() => actions.length).toBe(1)
  const body = actions[0].request().postData() ?? ""
  // Only the fields that changed are submitted; no removal flag is present.
  expect(submittedFields(body).removeAvatar).toBeUndefined()
  // The empty file input still submits an empty, unnamed file part. That part
  // must never be read as a removal.
  expect(body).toMatch(/name="avatar"; filename=""/u)
})

test("confirmed removal sends one explicit removal and no file part", async ({
  page
}) => {
  const actions: Route[] = []
  await openOwnEditor(page, actions)

  await page.getByRole("button", { name: REMOVE_LABEL, exact: true }).click()
  await page.locator("#editor button[type=submit]").click()
  await expect.poll(() => actions.length).toBe(1)

  const body = actions[0].request().postData() ?? ""
  expect(submittedFields(body).removeAvatar).toBe(true)
  expect(body).toMatch(/name="avatar"; filename=""/u)
})
