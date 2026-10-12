import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`

test("thread posts show, fold, expose options, and page in place without inline handlers", async ({
  page,
  request
}) => {
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(String(error)))
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  const response = await page.goto("/forum/t-17768232/national-fog-safety-initiative")
  expect(response?.status()).toBe(200)

  // ForumViewThreadPostsModule.initOnce reveals the hidden posts container.
  const posts = page.locator("#thread-container-posts")
  await expect(posts).toBeVisible()
  await expect(page.locator("#post-7000001 > .long")).toBeVisible()
  await expect(page.locator("#fpc-7000001 #post-7000002")).toBeVisible()

  // togglePostFold toggles .folded on the single post.
  await page.locator("#post-7000001 > .long .head .options a").click()
  await expect(page.locator("#post-7000001")).toHaveClass(/\bfolded\b/u)
  await expect(page.locator("#post-7000001 > .long")).toBeHidden()
  await expect(page.locator("#post-7000001 > .short")).toBeVisible()
  await expect(page.locator("#post-7000002")).not.toHaveClass(/\bfolded\b/u)
  await page.locator("#post-7000001 > .short a.options").click()
  await expect(page.locator("#post-7000001")).not.toHaveClass(/\bfolded\b/u)

  // Fold All / Unfold All act on every post.
  await page.getByRole("link", { name: "Fold All", exact: true }).click()
  await expect(page.locator("div.post.folded")).toHaveCount(2)
  await page.getByRole("link", { name: "Unfold All", exact: true }).click()
  await expect(page.locator("div.post.folded")).toHaveCount(0)

  // More Options reveals the thread options row and relabels the control.
  const toggle = page.locator("#thread-toggle-options")
  await toggle.click()
  await expect(page.locator("#thread-options-2")).toBeVisible()
  await expect(toggle).toHaveText("- Less Options")
  await toggle.click()
  await expect(page.locator("#thread-options-2")).toBeHidden()
  await expect(toggle).toHaveText("+ More Options")

  // Post options are filled from the template for that post.
  await page.locator("#post-7000002 > .long > .options a").click()
  const options = page.locator("#post-options-7000002")
  await expect(options).toBeVisible()
  await expect(options.locator("a")).toHaveAttribute(
    "onclick",
    "WIKIDOT.modules.ForumViewThreadModule.listeners.showPermalink(event,'7000002')"
  )

  // The pager reloads the posts in place through ForumViewThreadPostsModule.
  await posts
    .locator(".pager")
    .first()
    .getByRole("link", { name: "2", exact: true })
    .click()
  await expect(posts.locator(".pager-no").first()).toHaveText("page 2 of 2")
  await expect(page.locator("#post-7000011")).toBeVisible()
  await expect(page.locator("#post-7000001")).toHaveCount(0)
  await expect(page).toHaveURL(/\/forum\/t-17768232\/national-fog-safety-initiative$/u)

  const forumRequests = await request
    .get(`${FIXTURE_URL}/last-forum-module-requests`)
    .then((fixtureResponse) => fixtureResponse.json())
  expect(
    forumRequests.some(
      ({
        params
      }: {
        params: { module_name: string; parameters: Record<string, string> }
      }) =>
        params.module_name === "forum/ForumViewThreadPostsModule" &&
        params.parameters.t === "17768232" &&
        params.parameters.pageNo === "2"
    )
  ).toBe(true)
  expect(pageErrors).toEqual([])
})
