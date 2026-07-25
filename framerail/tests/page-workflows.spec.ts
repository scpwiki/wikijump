import { expect, test } from "@playwright/test"

import type { APIResponse } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token",
  origin: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
}
const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`

async function expectSuccessfulAction(response: APIResponse) {
  const body = await response.text()
  expect(response.ok(), body).toBe(true)
  expect(body).toContain('"type":"success"')
}

test("article routes carry load and mutation context through Deepwell", async ({
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-article-read-requests`)
  const article = await request.get("/page-workflow-probe", {
    headers: SITE_HEADERS
  })
  const cachedArticle = await request.get("/page-workflow-probe", {
    headers: SITE_HEADERS
  })
  expect(article.ok()).toBe(true)
  expect(article.headers()["x-content-type-options"]).toBe("nosniff")
  expect(cachedArticle.ok()).toBe(true)
  expect(await cachedArticle.text()).toBe(await article.text())
  expect(await article.text()).toContain("Page workflow probe")

  const loadRequests = await request
    .get(`${FIXTURE_URL}/last-article-read-requests`)
    .then((response) => response.json())
  const requestsForProbe = (requests: { route: { slug: string } }[]) =>
    requests.filter(({ route }) => route.slug === "page-workflow-probe")
  expect(requestsForProbe(loadRequests.articleView)).toHaveLength(1)
  expect(requestsForProbe(loadRequests.articleViewCacheMetadata)).toHaveLength(2)

  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/edit", {
      headers: AUTHENTICATED_HEADERS,
      multipart: {
        siteId: "6000005",
        pageId: "3000340",
        lastRevisionId: "9000340",
        title: "Page Workflow Probe",
        altTitle: "",
        wikitext: "Cross-layer edit",
        tags: "fixture",
        comments: "route edit"
      }
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/fileRestore", {
      headers: AUTHENTICATED_HEADERS,
      multipart: {
        siteId: "6000005",
        pageId: "3000340",
        lastRevisionId: "41",
        fileId: "42",
        newPage: "",
        newName: "",
        comments: "route file restore"
      }
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/voteCast", {
      headers: {
        ...AUTHENTICATED_HEADERS,
        "content-type": "text/plain;charset=UTF-8"
      },
      data: JSON.stringify({
        siteId: 6000005,
        pageId: 3000340,
        value: 1
      })
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/rollback", {
      headers: {
        ...AUTHENTICATED_HEADERS,
        "content-type": "text/plain;charset=UTF-8"
      },
      data: JSON.stringify({
        siteId: 6000005,
        pageId: 3000340,
        revisionNumber: 1,
        lastRevisionId: 9100000,
        comments: "route rollback"
      })
    })
  )

  const pageRequests = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  const fileRequests = await request
    .get(`${FIXTURE_URL}/last-file-requests`)
    .then((response) => response.json())

  expect(pageRequests.pageEdit).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        page: 3000340,
        wikitext: "Cross-layer edit"
      })
    })
  )
  expect(pageRequests.voteSet).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: {
        page_id: 3000340,
        user_id: 123,
        value: 1
      }
    })
  )
  expect(pageRequests.pageRollback).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        page: 3000340,
        revision_number: 1
      })
    })
  )
  expect(fileRequests.fileRestore).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        file_id: 42,
        page_id: 3000340
      })
    })
  )
})
