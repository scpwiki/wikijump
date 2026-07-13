import assert from "node:assert/strict"
import crypto from "node:crypto"
import fs from "node:fs/promises"
import test from "node:test"

const styles = [
  {
    file: "wikidot-base-c76c6921c8d6.css",
    sha256: "c76c6921c8d693044b78649a65fc7f1e0b775e5bbfc53cc01afd3098f1111128",
    source:
      "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--theme/base/css/style.css"
  },
  {
    file: "pagerate-db0bffe086ed.css",
    sha256: "db0bffe086ed2555bd90cb41737e79c67a6ed21d741f1eb116f7444e08e84403",
    source:
      "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/css/pagerate/PageRateWidgetModule.css"
  },
  {
    file: "sigma-fe5388a32e12.css",
    sha256: "fe5388a32e12934d38006694d6a64b66761990aaea536745773908bd0400edde",
    source: "https://cdn.scpwiki.com/theme/en/sigma/css/sigma.min.css"
  }
]

test("pinned Wikidot shell styles match their content-addressed filenames", async () => {
  for (const style of styles) {
    const contents = await fs.readFile(
      new URL(`../static/wikidot/styles/${style.file}`, import.meta.url)
    )
    assert.equal(crypto.createHash("sha256").update(contents).digest("hex"), style.sha256)
    assert.match(style.source, /^https:\/\//u)
  }
})

test("the Wikidot shell links only the pinned local copies", async () => {
  const layout = await fs.readFile(
    new URL("../src/routes/+layout.svelte", import.meta.url),
    "utf8"
  )

  const stylesheetHrefs = [
    ...layout.matchAll(/<link href="([^"]+)" rel="stylesheet" \/>/gu)
  ].map((match) => match[1])

  assert.deepEqual(
    stylesheetHrefs,
    styles.map((style) => `/wikidot/styles/${style.file}`)
  )
})

test("vendored Sigma CSS keeps every nested resource reference absolute", async () => {
  const sigma = await fs.readFile(
    new URL("../static/wikidot/styles/sigma-fe5388a32e12.css", import.meta.url),
    "utf8"
  )
  const urls = [...sigma.matchAll(/url\((['"]?)([^)'"\s]+)\1\)/giu)].map(
    (match) => match[2]
  )

  assert(urls.length > 0)
  assert(urls.every((url) => url.startsWith("https://") || url.startsWith("data:")))
})
