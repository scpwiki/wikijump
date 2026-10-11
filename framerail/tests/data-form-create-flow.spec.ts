import { expect, test, type Page } from "./hermetic-playwright"

const waitForHydration = async (page: Page) => {
  await page.waitForFunction(() => {
    const element = document.querySelector<HTMLAnchorElement>("#edit-save-button")
    if (!element) return false
    return Object.getOwnPropertySymbols(element).some((symbol) => {
      if (symbol.description !== "events") return false
      const handlers = (element as unknown as Record<symbol, { click?: unknown }>)[symbol]
      return typeof handlers?.click === "function"
    })
  })
}

const gotoHydratedEditor = async (page: Page, url: string) => {
  await page.goto(url)
  await waitForHydration(page)
}

const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`
const AUTHENTICATED_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki",
  cookie: "wikijump_token=fixture-session-token"
}

test("data-form create flow renders controls and stores Wikidot source", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  const anonymousMissingResponse = await page.goto("/data-form-create-flow:example")
  expect(anonymousMissingResponse?.status()).toBe(404)
  await expect(page.locator("[id='404-message']")).toHaveCount(1)
  await expect(page.locator("#restore-button")).toHaveCount(0)

  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const missingResponse = await page.goto("/data-form-create-flow:example")
  expect(missingResponse?.status()).toBe(404)
  await expect(page.locator("[id='404-message']")).toHaveCount(1)
  await expect(page.locator("#restore-button")).toHaveCount(0)
  await expect(page.locator("[id='404-message']")).toContainText(
    "data-form-create-flow:example"
  )
  await expect(page.locator("#create-it-now-link")).toHaveText("Create page")
  await page.locator("#create-it-now-link a").click()
  await expect(page).toHaveURL(/data-form-create-flow:example\/edit\/true$/u)
  await waitForHydration(page)

  await expect(page.locator("#action-area h1")).toHaveText("Create Data-form-create-flow")
  await expect(page.locator("textarea.debug")).toHaveCount(0)
  await expect(page.locator("#page-content")).toHaveCount(0)
  await expect(page.getByText("UNTRANSLATED:Page not found")).toHaveCount(0)
  await expect(page.locator("#edit-page-form")).toHaveClass("form-horizontal data-form")
  await expect(page.locator("input[name='form-use']")).toHaveValue("true")
  await expect(page.locator("input[name='form-fields']")).toHaveValue("name,choice")
  await expect(page.locator("#edit-page-title")).toHaveValue("Example")
  await expect(page.locator("input[name='field-name']")).toHaveValue("")
  await expect(page.locator("input[name='field-name']")).toHaveClass(
    "form-control form-text"
  )
  await expect(page.locator("input[name='field-choice'][value='b']")).toBeChecked()
  await expect(page.locator("input[name='field-choice'][value='a']")).toHaveClass(
    "form-select"
  )
  await expect(page.locator("input[name='field-choice'][value='b']")).toHaveClass(
    "form-select"
  )

  await page.locator("input[name='field-name']").press("Enter")
  const writesAfterEnter = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writesAfterEnter.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-create-flow:example"
    )
  ).toBeUndefined()

  await page.locator("input[name='field-name']").fill("Probe Name")
  await page.locator("input[name='field-choice'][value='a']").check()
  await page.locator("#edit-save-button").click()

  await expect(page).toHaveURL(/data-form-create-flow:104$/u)

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-create-flow:example"
      )
    })
    .toMatchObject({
      params: {
        slug: "data-form-create-flow:example",
        title: "Example",
        wikitext: "name: 'Probe Name'\nchoice: a",
        tags: []
      }
    })
})

test("data-form date field renders its basic control and preserves submitted scalars", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const missingResponse = await page.goto("/data-form-date-field-flow:example")
  expect(missingResponse?.status()).toBe(404)
  await page.locator("#create-it-now-link a").click()
  await expect(page).toHaveURL(/data-form-date-field-flow:example\/edit\/true$/u)
  await waitForHydration(page)
  const date = page.locator("input[name='field-date']")
  await expect(date).toHaveAttribute("type", "text")
  await expect(date).toHaveClass(/form-control form-date/u)
  await expect(date).toHaveAttribute("size", "24")
  await expect(date).toHaveClass(/hasDatepicker/u)
  const dateTrigger = page.locator("button.ui-datepicker-trigger")
  await expect(dateTrigger).toHaveCount(1)
  await dateTrigger.click()
  await expect(page.locator(".ui-datepicker")).toBeVisible()
  const today = page.locator(".ui-datepicker-calendar td.ui-datepicker-today a")
  await expect(today).toHaveCount(1)
  await today.click()
  await expect(date).toHaveValue(/^\d{2}\/\d{2}\/\d{4}$/u)
  await page.keyboard.press("Escape")

  await date.fill("02/29/2024")
  await page.locator("#edit-save-button").click()
  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string; wikitext?: string } }) =>
          entry.params.slug === "data-form-date-field-flow:example"
      )
    })
    .toMatchObject({ params: { wikitext: "date: 02/29/2024" } })
})

test("data-form datepicker selections save the evidenced Unix-second scalar", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-date-field-flow:example/edit/true")

  const date = page.locator("input[name='field-date']")
  const dateTrigger = page.locator("button.ui-datepicker-trigger")
  await dateTrigger.click()
  await expect(page.locator(".ui-datepicker")).toBeVisible()
  await page.locator(".ui-datepicker-calendar td.ui-datepicker-today a").click()
  await expect(date).toHaveValue(/^\d{2}\/\d{2}\/\d{4}$/u)
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string; wikitext?: string } }) =>
          entry.params.slug === "data-form-date-field-flow:example"
      )
    })
    .toMatchObject({ params: { wikitext: /^date: -?\d+$/u } })
})

test("pagepath Create new matches the immediate Wikidot tree mutation and survives form cancel", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-pagepath-flow:example/edit/true")

  const hidden = page.locator("input[name='field-origin']")
  await expect(hidden).toHaveClass("dataform-pagepath-value")
  await expect(hidden).toHaveValue("")
  await expect(page.locator("input.dataform-pagepath-category")).toHaveValue(
    "data-form-pagepath-tree"
  )
  await expect(page.locator("input.dataform-pagepath-max-level")).toHaveValue("3")

  const root = page.locator(
    "select.dataform-pagepath-select-children-of-data-form-pagepath-tree---_root"
  )
  await expect(root.locator("option")).toHaveText(["", "alpha", "Create new"])
  await root.selectOption("data-form-pagepath-tree:alpha")
  await expect(hidden).toHaveValue("data-form-pagepath-tree:alpha")

  const alpha = page.locator(
    "select.dataform-pagepath-select-children-of-data-form-pagepath-tree---alpha"
  )
  await expect(alpha.locator("option")).toHaveText(["", "beta", "Create new"])
  await alpha.selectOption("+")
  const newItem = page.locator(".dataform-pagepath-chooser input.text")
  await expect(newItem).toHaveValue("New item")
  await expect(
    page.locator(".dataform-pagepath-chooser a[href='javascript:;']")
  ).toHaveText("[x]")
  await expect(hidden).toHaveValue("data-form-pagepath-tree:alpha")

  await newItem.fill("gamma")
  await newItem.press("Enter")
  await expect(hidden).toHaveValue("data-form-pagepath-tree:gamma")
  await expect(alpha).toHaveValue("data-form-pagepath-tree:gamma")
  await expect(alpha.locator("option")).toHaveText(["", "beta", "gamma", "Create new"])
  await expect(
    page.locator(
      "select.dataform-pagepath-select-children-of-data-form-pagepath-tree---gamma option"
    )
  ).toHaveText(["", "Create new"])

  const writesAfterCreateNew = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writesAfterCreateNew.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-tree:gamma"
    )
  ).toMatchObject({
    params: {
      slug: "data-form-pagepath-tree:gamma",
      title: "gamma",
      wikitext: "",
      tags: []
    }
  })
  expect(
    writesAfterCreateNew.parentUpdate.find(
      (entry: { params: { child?: string } }) =>
        entry.params.child === "data-form-pagepath-tree:gamma"
    )
  ).toMatchObject({
    params: {
      child: "data-form-pagepath-tree:gamma",
      add: ["data-form-pagepath-tree:alpha"]
    }
  })
  expect(
    writesAfterCreateNew.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-flow:example"
    )
  ).toBeUndefined()

  await page.locator("#edit-cancel-button").click()
  const writesAfterCancel = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(writesAfterCancel.pageCreate).toEqual([])
  expect(
    writesAfterCancel.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-flow:example"
    )
  ).toBeUndefined()
  await page.goto("/data-form-pagepath-tree:gamma")
  await expect(page.locator("#page-title")).toHaveText("gamma")
})

test("pagepath first root child bootstraps _root and remains selected in the editor", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-pagepath-root-flow:example/edit/true")

  const hidden = page.locator("input[name='field-origin']")
  const root = page.locator(
    "select.dataform-pagepath-select-children-of-data-form-pagepath-root-tree---_root"
  )
  await expect(root.locator("option")).toHaveText(["", "Create new"])

  await root.selectOption("+")
  const newItem = page.locator(".dataform-pagepath-chooser input.text")
  await newItem.fill("alpha")
  await newItem.press("Enter")

  await expect(hidden).toHaveValue("data-form-pagepath-root-tree:alpha")
  await expect(root).toHaveValue("data-form-pagepath-root-tree:alpha")
  await expect(root.locator("option")).toHaveText(["", "alpha", "Create new"])
  await expect(
    page.locator(
      "select.dataform-pagepath-select-children-of-data-form-pagepath-root-tree---alpha option"
    )
  ).toHaveText(["", "Create new"])

  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-root-tree:_root"
    )
  ).toMatchObject({
    params: {
      slug: "data-form-pagepath-root-tree:_root",
      title: "Data-form-pagepath-root-tree",
      wikitext: "",
      tags: []
    }
  })
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-root-tree:alpha"
    )
  ).toMatchObject({
    params: {
      slug: "data-form-pagepath-root-tree:alpha",
      title: "alpha",
      wikitext: "",
      tags: []
    }
  })
  expect(
    writes.parentUpdate.find(
      (entry: { params: { child?: string } }) =>
        entry.params.child === "data-form-pagepath-root-tree:alpha"
    )
  ).toMatchObject({
    params: {
      child: "data-form-pagepath-root-tree:alpha",
      add: ["data-form-pagepath-root-tree:_root"]
    }
  })
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-pagepath-root-flow:example"
    )
  ).toBeUndefined()

  await page.locator("#edit-cancel-button").click()
  await page.goto("/data-form-pagepath-root-tree:alpha")
  await expect(page.locator("#page-title")).toHaveText("alpha")
})

test("date options edit reload formats stored seconds and serializes alt fields", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-date-options-flow:example/edit")

  const primary = page.locator("input[name='field-date-primary']")
  const secondary = page.locator("input[name='field-date-secondary']")
  const alt = page.locator("input[name='field-alt-date']")
  await expect(primary).toHaveValue("09/19/2014")
  await expect(secondary).toHaveValue("Friday, 12 December 2014")
  await expect(alt).toHaveValue("9/19/2014")
  await expect(page.locator("button.ui-datepicker-trigger")).toHaveCount(2)

  const expectedSelection = await page.evaluate(() => {
    const selected = new Date(2014, 8, 20)
    return {
      timestamp: String(Math.trunc(selected.getTime() / 1000)),
      alt: "9/20/2014"
    }
  })
  await page.locator("button.ui-datepicker-trigger").first().click()
  await expect(page.locator(".ui-datepicker")).toBeVisible()
  await page.locator(".ui-datepicker-calendar a[data-date='20']").click()
  await expect(alt).toHaveValue(expectedSelection.alt)
  await page.locator("#edit-save-button").click()

  await expect(page.locator("#edit-page-form")).toHaveCount(0)
  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writes.pageEdit.find(
      (entry: { params: { page?: number } }) => entry.params.page === 3000400
    )
  ).toMatchObject({
    params: {
      page: 3000400,
      last_revision_id: 9000400,
      wikitext: `date-primary: ${expectedSelection.timestamp}\ndate-secondary: 1418360400\nalt-date: '${expectedSelection.alt}'`
    }
  })
})

test("data-form text and select controls match live validation and storage", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-controls-flow:example/edit/true")

  const plain = page.locator("input[name='field-plain']")
  await expect(plain).toHaveAttribute("size", "1")
  await expect(plain).toHaveValue("**bold** #hash")
  const multi = page.locator("textarea[name='field-multi']")
  await expect(multi).toHaveAttribute("rows", "3")
  await expect(multi).toHaveAttribute("cols", "50")
  const matched = page.locator("input[name='field-matched']")
  await expect(matched).toHaveAttribute("size", "40")
  await expect(matched).toHaveAttribute("placeholder", "enter a color like \\#468259")
  await expect(page.locator("input[name='field-select_one']")).toHaveCount(1)
  await expect(page.locator("input[name='field-select_one']")).not.toBeChecked()
  await expect(page.locator("input[name='field-select_four']")).toHaveCount(4)
  await expect(page.locator("input[name='field-select_four'][value='c']")).toBeChecked()
  const selectFive = page.locator("select[name='field-select_five']")
  await expect(selectFive).toHaveValue("4")
  await expect(selectFive.locator("option")).toHaveText([
    "Zero",
    "One",
    "Two",
    "Three",
    "Four"
  ])

  await plain.fill(`O'Brien: # [x] \\ slash "quote"`)
  await multi.fill(`first "quoted"\nsecond 'single' \\ end`)
  await matched.fill("bad")
  await page.locator("input[name='field-select_one'][value='a']").check()
  await selectFive.selectOption("2")
  await page.locator("#edit-save-button").click()

  const matchedGroup = matched.locator(
    "xpath=ancestor::div[contains(concat(' ', normalize-space(@class), ' '), ' form-group ')][1]"
  )
  await expect(matchedGroup).toHaveClass(/(?:^|\s)has-error(?:\s|$)/u)
  await expect(matched.locator("xpath=..")).toHaveClass(/(?:^|\s)form-error(?:\s|$)/u)
  await expect(matchedGroup.locator(".form-message")).toHaveClass(
    /(?:^|\s)text-danger(?:\s|$)/u
  )
  await expect(matchedGroup.locator(".form-message")).toHaveText(
    "Use ok- followed by digits"
  )
  await expect(plain).toHaveValue(`O'Brien: # [x] \\ slash "quote"`)
  await expect(multi).toHaveValue(`first "quoted"\nsecond 'single' \\ end`)
  const writesAfterInvalid = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writesAfterInvalid.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-controls-flow:example"
    )
  ).toBeUndefined()

  await matched.fill("ok-42")
  await page.locator("#edit-save-button").click()
  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-controls-flow:example"
      )
    })
    .toMatchObject({
      params: {
        slug: "data-form-controls-flow:example",
        wikitext: [
          "plain: 'O''Brien: # [x] \\ slash \"quote\"'",
          'multi: "first \\"quoted\\"\\nsecond \'single\' \\\\ end"',
          "matched: ok-42",
          "select_one: a",
          "select_four: c",
          "select_five: '2'"
        ].join("\n"),
        tags: []
      }
    })

  await gotoHydratedEditor(page, "/data-form-controls-flow:example/edit")
  await expect(page.locator("input[name='field-plain']")).toHaveValue(
    `O'Brien: # [x] \\ slash "quote"`
  )
  await expect(page.locator("textarea[name='field-multi']")).toHaveValue(
    `first "quoted"\nsecond 'single' \\ end`
  )
  await expect(page.locator("input[name='field-matched']")).toHaveValue("ok-42")
  await expect(page.locator("input[name='field-select_one'][value='a']")).toBeChecked()
  await expect(page.locator("input[name='field-select_four'][value='c']")).toBeChecked()
  await expect(page.locator("select[name='field-select_five']")).toHaveValue("2")
})

test("data-form checkbox and wiki controls match live DOM and storage", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-checkbox-wiki-flow:example/edit/true")

  const unchecked = page.locator("input[name='field-checkbox_unchecked']")
  await expect(unchecked).toHaveAttribute("type", "checkbox")
  await expect(unchecked).toHaveClass("form-checkbox")
  await expect(unchecked).not.toHaveAttribute("value")
  await expect(unchecked).not.toBeChecked()
  expect(await unchecked.inputValue()).toBe("on")

  const checked = page.locator("input[name='field-checkbox_checked']")
  await expect(checked).toBeChecked()
  await expect(checked).toHaveAttribute("checked", "checked")
  await expect(checked).not.toHaveAttribute("value")
  await expect(page.locator(".field-checkbox_unchecked")).toContainText("PRE POST")
  await expect(unchecked).not.toHaveAttribute("placeholder")
  expect(
    await page
      .locator(".field-checkbox_unchecked")
      .evaluate((element) =>
        [...element.childNodes]
          .filter((node) => node.nodeType !== Node.COMMENT_NODE)
          .map((node) =>
            node.nodeType === Node.TEXT_NODE ? node.textContent : node.nodeName
          )
      )
  ).toEqual(["PRE ", "INPUT", " POST", "SPAN"])

  const wiki = page.locator("textarea[name='field-wiki']")
  await expect(wiki).toHaveClass("form-control form-wiki")
  await expect(wiki).toHaveAttribute("cols", "40")
  await expect(wiki).toHaveAttribute("rows", "2")
  await expect(wiki).toHaveAttribute("placeholder", "enter wiki \\#source")
  await expect(wiki).toHaveValue("**Default**")
  await expect(page.locator(".field-wiki")).toContainText("**Before** //After//")
  expect(
    await page
      .locator(".field-wiki")
      .evaluate((element) =>
        [...element.childNodes]
          .filter((node) => node.nodeType !== Node.COMMENT_NODE)
          .map((node) =>
            node.nodeType === Node.TEXT_NODE ? node.textContent : node.nodeName
          )
      )
  ).toEqual(["**Before** ", "TEXTAREA", " //After//", "SPAN"])

  const wikiOneLine = page.locator("input[name='field-wiki_one_line']")
  await expect(wikiOneLine).toHaveClass("form-control form-wiki")
  await expect(wikiOneLine).toHaveAttribute("size", "20")

  await unchecked.check()
  await checked.uncheck()
  await wiki.fill("**Bold**\n[[[start|Home]]]")
  await wikiOneLine.fill("//italic//")
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-checkbox-wiki-flow:example"
      )
    })
    .toMatchObject({
      params: {
        wikitext: [
          "checkbox_unchecked: '1'",
          "checkbox_checked: '0'",
          'wiki: "**Bold**\\n[[[start|Home]]]"',
          "wiki_one_line: //italic//"
        ].join("\n")
      }
    })
})

test("empty and unselected select fields save and restore as Wikidot null", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-empty-select-flow:example/edit/true")

  await expect(page.locator("[name='field-missing_values']")).toHaveCount(0)
  await expect(page.locator("[name='field-empty_values']")).toHaveCount(0)
  await expect(page.locator("input[name='field-select_one']")).not.toBeChecked()
  const selectTwo = page.locator("input[name='field-select_two']")
  await expect(selectTwo).toHaveCount(2)
  expect(
    await selectTwo.evaluateAll((controls) =>
      controls.every((control) => !(control as HTMLInputElement).checked)
    )
  ).toBe(true)
  await expect(page.locator("select[name='field-select_five']")).toHaveValue("a")
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-empty-select-flow:example"
      )
    })
    .toMatchObject({
      params: {
        wikitext: [
          "missing_values: null",
          "empty_values: null",
          "select_one: null",
          "select_two: null",
          "select_five: a"
        ].join("\n")
      }
    })

  await gotoHydratedEditor(page, "/data-form-empty-select-flow:example/edit")
  await expect(page.locator("[name='field-missing_values']")).toHaveCount(0)
  await expect(page.locator("[name='field-empty_values']")).toHaveCount(0)
  await expect(page.locator("input[name='field-select_one']")).not.toBeChecked()
  await expect(selectTwo).toHaveCount(2)
  expect(
    await selectTwo.evaluateAll((controls) =>
      controls.every((control) => !(control as HTMLInputElement).checked)
    )
  ).toBe(true)
  await expect(page.locator("select[name='field-select_five']")).toHaveValue("a")
})

test("data-form field properties and PCRE-style match behavior follow live Wikidot", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-properties-flow:example/edit/true")

  const base = page.locator("input[name='field-base']")
  const joined = page.locator("input[name='field-joined']")
  const baseGroup = base.locator(
    "xpath=ancestor::div[contains(concat(' ', normalize-space(@class), ' '), ' form-group ')][1]"
  )
  const joinedGroup = joined.locator(
    "xpath=ancestor::div[contains(concat(' ', normalize-space(@class), ' '), ' form-group ')][1]"
  )
  await expect(baseGroup.locator("input[name='field-joined']")).toHaveCount(1)
  await expect(joinedGroup.locator("input[name='field-base']")).toHaveCount(1)
  await expect(baseGroup.locator(":scope > .control-label")).toHaveText("Base label")
  await expect(baseGroup.locator(":scope > .control-label")).not.toHaveAttribute("for")
  await expect(base).not.toHaveAttribute("id")
  expect(
    await page
      .locator(".field-base")
      .evaluate((element) =>
        [...element.childNodes]
          .filter((node) => node.nodeType !== Node.COMMENT_NODE)
          .map((node) =>
            node.nodeType === Node.TEXT_NODE ? node.textContent : node.nodeName
          )
      )
  ).toEqual([" ", "INPUT", " ", "SPAN"])
  expect(
    await page
      .locator(".field-joined")
      .evaluate((element) =>
        [...element.childNodes]
          .filter((node) => node.nodeType !== Node.COMMENT_NODE)
          .map((node) =>
            node.nodeType === Node.TEXT_NODE ? node.textContent : node.nodeName
          )
      )
  ).toEqual(["PRE ", "INPUT", " POST", "SPAN"])
  await expect(baseGroup.locator(":scope > .col-sm-5")).toContainText(
    "Joined labelPRE POST"
  )
  await expect(page.locator(".field-joined")).toContainText("PRE POST")

  const extended = page.locator("textarea[name='field-extended']")
  await expect(extended).toHaveAttribute("placeholder", "  padded # hint  ")
  await expect(page.locator(".field-extended")).toContainText("pre # post")
  await expect(
    page
      .locator(".field-extended")
      .locator("xpath=ancestor::div[contains(@class, 'form-group')][1]")
      .locator(":scope > .control-label")
  ).toHaveText("")

  const choice = page.locator("input[name='field-choice']")
  await expect(choice).toHaveCount(2)
  await expect(choice.first()).not.toHaveAttribute("placeholder")
  await expect(page.locator(".field-choice")).toContainText("PRE AlphaBeta POST")

  await base.fill("base value")
  await joined.fill("bad")
  await extended.fill("ab")
  await page.locator("input[name='field-duplicate_modifier']").fill("ok")
  await page.locator("input[name='field-choice'][value='b']").check()
  await page.locator("#edit-save-button").click()

  await expect(joinedGroup).toHaveClass(/(?:^|\s)has-error(?:\s|$)/u)
  await expect(joined.locator("xpath=..")).toHaveClass(/(?:^|\s)form-error(?:\s|$)/u)
  await expect(joined.locator("xpath=..").locator(".form-message")).toHaveText(
    "Please enter valid 'Joined label'"
  )
  const writesAfterInvalid = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writesAfterInvalid.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-properties-flow:example"
    )
  ).toBeUndefined()

  await joined.fill("OK")
  await page.locator("#edit-save-button").click()
  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-properties-flow:example"
      )
    })
    .toMatchObject({
      params: {
        slug: "data-form-properties-flow:example",
        wikitext: [
          "base: 'base value'",
          "joined: OK",
          "extended: ab",
          "duplicate_modifier: ok",
          "choice: b"
        ].join("\n")
      }
    })
})

test("pathological data-form match patterns fail closed without blocking the editor", async ({
  page,
  request
}) => {
  test.setTimeout(15_000)
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-regex-budget-flow:example/edit/true")

  const matched = page.locator("input[name='field-matched']")
  const matchedTwo = page.locator("input[name='field-matched_two']")
  await matched.fill(`${"a".repeat(28)}!`)
  await matchedTwo.fill(`${"a".repeat(28)}!`)
  const validationStartedAt = Date.now()
  await page.locator("#edit-save-button").click()
  await expect(page.locator(".form-message")).toHaveText([
    "Wikijump could not safely evaluate this field.",
    "Wikijump could not safely evaluate this field."
  ])
  expect(Date.now() - validationStartedAt).toBeLessThan(450)

  await page.locator("#edit-page-title").fill("Editor remains responsive")
  await expect(page.locator("#edit-page-title")).toHaveValue("Editor remains responsive")
  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-regex-budget-flow:example"
    )
  ).toBeUndefined()
})

test("a new data-form submission cancels stale validation and saves its own snapshot", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-regex-budget-flow:example/edit/true")

  const matched = page.locator("input[name='field-matched']")
  const matchedTwo = page.locator("input[name='field-matched_two']")
  await page.locator("#edit-page-title").fill("Stale snapshot")
  await matched.fill(`${"a".repeat(28)}!`)
  await matchedTwo.fill(`${"a".repeat(28)}!`)
  await page.locator("#edit-save-button").click()

  await page.locator("#edit-page-title").fill("Current snapshot")
  await matched.fill("a")
  await matchedTwo.fill("aa")
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageCreate.find(
        (entry: { params: { slug?: string } }) =>
          entry.params.slug === "data-form-regex-budget-flow:example"
      )
    })
    .toMatchObject({
      params: {
        title: "Current snapshot",
        wikitext: "matched: a\nmatched_two: aa"
      }
    })
})

test("invalid data-form match patterns use a host-owned diagnostic", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-invalid-regex-flow:example/edit/true")

  await page.locator("input[name='field-matched']").fill("anything")
  await page.locator("#edit-save-button").click()
  await expect(page.locator(".form-message")).toHaveText(
    "Wikijump could not evaluate this field's validation pattern."
  )

  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-invalid-regex-flow:example"
    )
  ).toBeUndefined()
})

test("data-form edit flow restores and updates saved field values", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(page, "/data-form-edit-flow:example/edit")

  await expect(page.locator("#action-area h1")).toHaveText("Edit Data-form-edit-flow")
  await expect(page.locator("#page-content")).toBeHidden()
  await expect(page.locator("input[name='field-name']")).toHaveValue("Probe Name")
  await expect(page.locator("input[name='field-choice'][value='a']")).toBeChecked()

  await page.locator("input[name='field-name']").fill("Updated Name")
  await page.locator("input[name='field-choice'][value='b']").check()
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.pageEdit.find(
        (entry: { params: { page?: number } }) => entry.params.page === 3000420
      )
    })
    .toMatchObject({
      params: {
        page: 3000420,
        last_revision_id: 9000420,
        title: "Example",
        wikitext: "name: 'Updated Name'\nchoice: b",
        tags: []
      }
    })
})

test("data-form create rejects NewPage tags without creating a page", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(
    page,
    "/data-form-create-flow:example/edit/true/tags/rock%20live"
  )

  await page.locator("input[name='field-name']").fill("Tagged Value")
  await page.locator("input[name='field-choice'][value='a']").check()
  await page.locator("#edit-save-button").click()

  await expect(
    page.getByText("An error occurred while processing the request.")
  ).toBeVisible()
  await expect(page.locator("#edit-page-form")).toBeVisible()
  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(
    writes.pageCreate.find(
      (entry: { params: { slug?: string } }) =>
        entry.params.slug === "data-form-create-flow:example"
    )
  ).toBeUndefined()
})

test("data-form create applies the NewPage parent after saving", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await gotoHydratedEditor(
    page,
    "/data-form-create-flow:example/edit/true/parentPage/data-form-create-flow%3Aparent"
  )

  await page.locator("input[name='field-name']").fill("Parent Value")
  await page.locator("input[name='field-choice'][value='a']").check()
  await page.locator("#edit-save-button").click()

  await expect
    .poll(async () => {
      const writes = await request
        .get(`${FIXTURE_URL}/last-page-write-requests`)
        .then((response) => response.json())
      return writes.parentUpdate.find(
        (entry: { params: { child?: string } }) =>
          entry.params.child === "data-form-create-flow:104"
      )
    })
    .toMatchObject({
      params: {
        child: "data-form-create-flow:104",
        add: ["data-form-create-flow:parent"],
        remove: []
      }
    })
})
