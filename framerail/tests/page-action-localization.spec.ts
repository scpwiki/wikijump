import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const HTTPS_APP_URL = `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`

const primaryActions = [
  ["edit-button", "edit"],
  ["pagerate-button", "vote"],
  ["history-button", "history"],
  ["files-button", "files"]
] as const

const expandedActions = [
  ["edit-append-button", "append"],
  ["edit-sections-button", "edit-sections"],
  ["edit-meta-button", "edit-meta"],
  ["watchers-button", "watchers"],
  ["backlinks-button", "backlinks"],
  ["view-source-button", "view-source"],
  ["layout-button", "layout"],
  ["parent-page-button", "parents"],
  ["lock-page-button", "lock"],
  ["rename-move-button", "rename-move"],
  ["delete-button", "delete"]
] as const

const labels = {
  "en-US": {
    edit: "Edit",
    vote: "Vote",
    history: "History",
    files: "Files",
    options: "Options",
    append: "Append",
    "edit-sections": "Edit Sections",
    "edit-meta": "Edit Meta",
    watchers: "Watchers",
    backlinks: "Backlinks",
    "view-source": "View Source",
    layout: "Layout",
    parents: "Parents",
    lock: "Lock Page",
    "rename-move": "Move",
    delete: "Delete"
  },
  "ja-JP": {
    edit: "編集",
    vote: "評価",
    history: "履歴",
    files: "ファイル",
    options: "オプション",
    append: "追加",
    "edit-sections": "セクションを編集",
    "edit-meta": "メタを編集",
    watchers: "ウォッチャー",
    backlinks: "バックリンク",
    "view-source": "ページソース",
    layout: "レイアウト",
    parents: "親ページ",
    lock: "ページロック",
    "rename-move": "リネーム",
    delete: "削除"
  }
} as const

for (const [locale, expected] of Object.entries(labels) as Array<
  [keyof typeof labels, (typeof labels)[keyof typeof labels]]
>) {
  test.describe(`page actions expose localized names (${locale})`, () => {
    test.use({ locale })

    test("primary and expanded actions are keyboard-accessible at 320px", async ({
      page
    }, testInfo) => {
      test.skip(
        testInfo.project.name === "webkit",
        "WebKit runs this acceptance case against the HTTPS fixture"
      )

      await page.setViewportSize({ width: 320, height: 640 })
      await page.setExtraHTTPHeaders(SITE_HEADERS)
      const baseUrl =
        testInfo.project.name === "webkit-https-edit-meta" ? HTTPS_APP_URL : APP_URL
      const response = await page.goto(`${baseUrl}/scp-173`)
      expect(response?.status()).toBe(200)

      await waitForSvelteDelegatedHandler(page, "#more-options-button")
      for (const [id, key] of primaryActions) {
        const action = page.locator(`#${id}`)
        await expect(action).toBeVisible()
        await expect(action).toHaveAccessibleName(expected[key])
      }

      const moreOptions = page.locator("#more-options-button")
      await moreOptions.focus()
      await expect(moreOptions).toBeFocused()
      await page.keyboard.press("Enter")
      await expect(page.locator("#page-options-bottom-2")).toBeVisible()

      for (const [id, key] of expandedActions) {
        const action = page.locator(`#${id}`)
        await expect(action).toBeVisible()
        await expect(action).toHaveAccessibleName(expected[key])
      }

      const firstExpandedAction = page.locator("#edit-append-button")
      await firstExpandedAction.focus()
      await expect(firstExpandedAction).toBeFocused()
      await page.keyboard.press("Tab")
      await expect(page.locator("#edit-sections-button")).toBeFocused()

      const writes = await page.request.get(
        `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}/last-page-write-requests`
      )
      expect(writes.ok()).toBe(true)
      const recorded = (await writes.json()) as Record<string, unknown[]>
      expect(Object.values(recorded).every((requests) => requests.length === 0)).toBe(
        true
      )
    })
  })
}
