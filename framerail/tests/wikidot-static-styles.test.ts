import assert from "node:assert/strict"
import crypto from "node:crypto"
import fs from "node:fs/promises"
import test from "node:test"

const styles = [
  {
    file: "wikidot-base-165bc434fd1d.css",
    sha256: "165bc434fd1da2092fee0ea6bdeb55aa38402aaaafd6d1e3303180d2b595b981",
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

test("the shell wrapper leaves imported page themes in control of typography", async () => {
  const layout = await fs.readFile(
    new URL("../src/lib/sigma-esque/wikidot.svelte", import.meta.url),
    "utf8"
  )
  const wrapperRule = /#skrollr-body\s*\{(?<declarations>[^}]*)\}/u.exec(layout)

  assert.doesNotMatch(
    wrapperRule?.groups?.declarations ?? "",
    /(?:--font-|font-|line-height|text-rendering)/u
  )
})

test("the modern top bar styles cannot match imported Wikidot navigation", async () => {
  const layout = await fs.readFile(
    new URL("../src/lib/sigma-esque/sigma-esque.svelte", import.meta.url),
    "utf8"
  )

  assert.match(layout, /\.sigma-esque-container\s*>\s*\.top-bar\s*\{/u)
  assert.doesNotMatch(layout, /^\s*\.top-bar\s*\{/mu)
})

test("the modern page-tag layout cannot override imported Wikidot theme CSS", async () => {
  const page = await fs.readFile(
    new URL("../src/routes/[slug]/[...extra]/page.svelte", import.meta.url),
    "utf8"
  )

  assert.match(page, /\.sigma-esque-container\s+\.page-tags\s*\{/u)
  assert.doesNotMatch(page, /^\s*\.page-tags\s*\{/mu)
})

test("the Wikidot shell preserves the legacy two-input search chrome", async () => {
  const layout = await fs.readFile(
    new URL("../src/routes/+layout.svelte", import.meta.url),
    "utf8"
  )

  assert.match(layout, /<div id="search-top-box">/u)
  assert.match(layout, /<form id="search-top-box-form" action="dummy">/u)
  assert.match(layout, /id="search-top-box-input"[\s\S]*?type="text"/u)
  assert.match(
    layout,
    /<input(?=[^>]*name="search")(?=[^>]*class="button")(?=[^>]*type="submit")(?=[^>]*value="Search")[^>]*>/u
  )
})

test("the Wikidot error dialog exposes the real visible display state", async () => {
  const popup = await fs.readFile(
    new URL("../src/lib/popup/error.svelte", import.meta.url),
    "utf8"
  )

  assert.match(popup, /id="odialog-container"\s+style:display="block"/u)
  assert.doesNotMatch(popup, /basalt-compat/u)
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

test("vendored Wikidot base CSS resolves its YUI sprite from the pinned source", async () => {
  const base = await fs.readFile(
    new URL("../static/wikidot/styles/wikidot-base-165bc434fd1d.css", import.meta.url),
    "utf8"
  )

  assert.doesNotMatch(base, /url\(\.\.\/\.\.\/\.\.\/common--javascript\//u)
  assert.equal(
    base.match(
      /https:\/\/d3g0gp89917ko0\.cloudfront\.net\/v--3b8418686296\/common--javascript\/yahooui\/assets\/sprite\.png/gu
    )?.length,
    3
  )
})
