import { expect, test } from "./hermetic-playwright"

const siteChangesMarkup = `
  <div class="site-changes-box">
    <form onsubmit="return false;" action="dummy.html" method="get">
      <input type="checkbox" id="rev-type-all" checked="checked" />
      <input type="checkbox" id="rev-type-new" />
      <input type="checkbox" id="rev-type-source" />
      <input type="checkbox" id="rev-type-title" />
      <input type="checkbox" id="rev-type-move" />
      <input type="checkbox" id="rev-type-tags" />
      <input type="checkbox" id="rev-type-meta" />
      <input type="checkbox" id="rev-type-files" />
      <select id="rev-category">
        <option value="">Whole site</option>
        <option value="100000011">nav</option>
      </select>
      <select id="rev-perpage">
        <option value="10">10</option>
        <option value="20" selected="selected">20</option>
      </select>
      <input type="button" value="Update list" onclick="WIKIDOT.modules.SiteChangesModule.listeners.updateList(null)" />
    </form>
    <div class="changes-list" id="site-changes-list">
      <div class="changes-list-item">system:old result</div>
      <div class="changes-list-item">_default:old result</div>
    </div>
  </div>
`

test("SiteChanges Update list refreshes the selected category and preserves controls", async ({
  page
}) => {
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  let requestCount = 0
  let submittedFields: URLSearchParams | undefined
  await page.route("**/ajax-module-connector.php", async (route) => {
    requestCount += 1
    submittedFields = new URLSearchParams(route.request().postData() ?? "")
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        status: "ok",
        body: '<div class="changes-list-item">nav: Top Navigation</div><div class="changes-list-item">nav: Side Navigation</div>'
      })
    })
  })

  await page.goto("/wikidot-tabview", { waitUntil: "networkidle" })
  await page.locator("#page-content").evaluate((content, markup) => {
    content.innerHTML = markup
  }, siteChangesMarkup)
  await page.locator("#rev-category").selectOption("100000011")
  await page.getByRole("button", { name: "Update list" }).click()

  expect(requestCount).toBe(1)
  expect(submittedFields?.toString()).toContain("categoryId=100000011")
  const rows = page.locator("#site-changes-list .changes-list-item")
  await expect(rows).toHaveText(["nav: Top Navigation", "nav: Side Navigation"])
  expect(submittedFields?.get("moduleName")).toBe("changes/SiteChangesListModule")
  expect(submittedFields?.get("page")).toBe("1")
  expect(submittedFields?.get("pageId")).toMatch(/^[1-9][0-9]*$/u)
  expect(submittedFields?.get("categoryId")).toBe("100000011")
  expect(submittedFields?.get("perpage")).toBe("20")
  expect(submittedFields?.get("options")).toBe('{"all":true}')
  await expect(page.locator("#rev-category")).toHaveValue("100000011")
  await expect(page.locator("#rev-perpage")).toHaveValue("20")
  await expect(page.locator("#rev-type-all")).toBeChecked()
})
