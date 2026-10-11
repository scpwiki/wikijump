import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

let vite
let buildPageTranslateKeys
let getPreloadBackendLocales

before(async () => {
  vite = await createTestViteServer()
  ;({ buildPageTranslateKeys } = await vite.ssrLoadModule(
    "/src/lib/server/load/page/page-presentation.ts"
  ))
  ;({ getPreloadBackendLocales } = await vite.ssrLoadModule(
    "/src/lib/server/load/preload.ts"
  ))
})

after(async () => {
  if (vite) await vite.close()
})

const pageActionKeys = [
  "edit",
  "vote",
  "history",
  "files",
  "options",
  "layout",
  "parents",
  "delete",
  "wiki-page-view-source",
  "wiki-page-action-append",
  "wiki-page-action-edit-sections",
  "wiki-page-action-edit-meta",
  "wiki-page-action-watchers",
  "wiki-page-action-backlinks",
  "wiki-page-action-lock",
  "wiki-page-action-rename-move"
]

test("page presentation requests every primary and expanded action label", () => {
  const keys = buildPageTranslateKeys(
    {
      type: "found",
      data: {
        page: { created_at: "2026-10-01T00:00:00.000Z" },
        page_revision: { revision_number: 1 }
      }
    },
    {
      licenseName: "CC BY-SA",
      licenseUrl: "https://example.test/license",
      locales: ["ja"],
      now: 0
    }
  )

  for (const key of pageActionKeys) assert.ok(Object.hasOwn(keys, key), key)
})

test("English fallback and Japanese catalogs define the exact page action labels", async () => {
  const baseEn = await readFile(
    new URL("../../locales/fluent/base/en.ftl", import.meta.url),
    "utf8"
  )
  const baseJa = await readFile(
    new URL("../../locales/fluent/base/ja.ftl", import.meta.url),
    "utf8"
  )
  const wikiEn = await readFile(
    new URL("../../locales/fluent/wiki-page/en.ftl", import.meta.url),
    "utf8"
  )
  const wikiJa = await readFile(
    new URL("../../locales/fluent/wiki-page/ja.ftl", import.meta.url),
    "utf8"
  )
  const wikiZhHans = await readFile(
    new URL("../../locales/fluent/wiki-page/zh_Hans.ftl", import.meta.url),
    "utf8"
  )

  const baseLabels = {
    delete: "削除",
    edit: "編集",
    files: "ファイル",
    history: "履歴",
    layout: "レイアウト",
    options: "オプション",
    parents: "親ページ",
    vote: "評価"
  }
  const pageLabels = {
    "wiki-page-view-source": "ページソース",
    "wiki-page-action-append": "追加",
    "wiki-page-action-edit-sections": "セクションを編集",
    "wiki-page-action-edit-meta": "メタを編集",
    "wiki-page-action-watchers": "ウォッチャー",
    "wiki-page-action-backlinks": "バックリンク",
    "wiki-page-action-lock": "ページロック",
    "wiki-page-action-rename-move": "リネーム"
  }
  for (const key of Object.keys(baseLabels)) {
    assert.match(baseEn, new RegExp(`^${key} = .+$`, "mu"), `en: ${key}`)
    assert.match(baseJa, new RegExp(`^${key} = ${baseLabels[key]}$`, "mu"), `ja: ${key}`)
  }
  for (const [key, label] of Object.entries(pageLabels)) {
    assert.match(wikiEn, new RegExp(`^${key} = .+$`, "mu"), `en: ${key}`)
    assert.match(wikiJa, new RegExp(`^${key} = ${label}$`, "mu"), `ja: ${key}`)
  }
  assert.match(wikiZhHans, /^wiki-page-action-lock = 锁定页面$/mu)
  assert.match(wikiZhHans, /^wiki-page-action-rename-move = 移动$/mu)
})

test("non-Japanese page locales retain the English fallback after their preferred locale", () => {
  assert.deepEqual(getPreloadBackendLocales(["ja"]), ["ja", "en"])
  assert.deepEqual(getPreloadBackendLocales(["zh-Hans"]), ["zh-Hans", "en"])
})

test("visible page action text remains the anchor accessible name", async () => {
  const pageView = await readFile(
    new URL("../src/routes/[slug]/[...extra]/PageView.svelte", import.meta.url),
    "utf8"
  )
  const actionKeys = {
    "edit-button": "internationalization?.edit",
    "pagerate-button": "internationalization?.vote",
    "history-button": "internationalization?.history",
    "files-button": "internationalization?.files",
    "more-options-button": "internationalization?.options",
    "edit-append-button": "wiki-page-action-append",
    "edit-sections-button": "wiki-page-action-edit-sections",
    "edit-meta-button": "wiki-page-action-edit-meta",
    "watchers-button": "wiki-page-action-watchers",
    "backlinks-button": "wiki-page-action-backlinks",
    "view-source-button": "wiki-page-view-source",
    "layout-button": "internationalization?.layout",
    "parent-page-button": "internationalization?.parents",
    "lock-page-button": "wiki-page-action-lock",
    "rename-move-button": "wiki-page-action-rename-move",
    "delete-button": "internationalization?.delete"
  }

  for (const [id, labelExpression] of Object.entries(actionKeys)) {
    const start = pageView.indexOf(`id="${id}"`)
    assert.ok(start >= 0, id)
    const end = pageView.indexOf("</a>", start)
    assert.ok(end >= start, id)
    assert.ok(pageView.slice(start, end).includes(labelExpression), id)
  }
})
