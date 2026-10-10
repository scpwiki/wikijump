import assert from "node:assert/strict"
import crypto from "node:crypto"
import fs from "node:fs/promises"
import test from "node:test"

import postcss from "postcss"
import scss from "postcss-scss"

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

const baseAssets = [
  {
    file: "common--theme/base/images/shade2_n.png",
    sha256: "2b3f53a407d5b25bc91bd9920f164b13e14d944bc95a7bf32a5138b30cef07c6",
    source:
      "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--theme/base/images/shade2_n.png"
  },
  {
    file: "common--theme/base/images/feed/feed-icon-14x14.png",
    sha256: "8ee173565b2e771fecf3b471a79bdf072aaa1bd9dc27582cfda2b2a322beeba8",
    source:
      "https://d3g0gp89917ko0.cloudfront.net/v--0c0da3649c4f/common--theme/base/images/feed/feed-icon-14x14.png"
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

test("vendored Wikidot base assets match their pinned sources", async () => {
  for (const asset of baseAssets) {
    const contents = await fs.readFile(
      new URL(`../static/${asset.file}`, import.meta.url)
    )
    assert.equal(crypto.createHash("sha256").update(contents).digest("hex"), asset.sha256)
    assert.match(asset.source, /^https:\/\//u)
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

test("modern typography is scoped away from the imported Wikidot shell", async () => {
  const [baseTypography, sigmaShell] = await Promise.all([
    fs.readFile(new URL("../src/lib/css/base/_typography.scss", import.meta.url), "utf8"),
    fs.readFile(
      new URL("../src/lib/sigma-esque/sigma-esque.svelte", import.meta.url),
      "utf8"
    )
  ])

  assert.match(baseTypography, /\.sigma-esque-container\s*\{/u)
  assert.doesNotMatch(baseTypography, /(?:^|[,{])\s*(?:html|body)\s*\{/mu)
  assert.match(sigmaShell, /body:has\(\.sigma-esque-container\)\s*\{/u)
  assert.doesNotMatch(sigmaShell, /^\s*body\s*\{/mu)
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
  const [page, pageStyles] = await Promise.all([
    fs.readFile(
      new URL("../src/routes/[slug]/[...extra]/PageView.svelte", import.meta.url),
      "utf8"
    ),
    fs.readFile(
      new URL("../src/routes/[slug]/[...extra]/page.scss", import.meta.url),
      "utf8"
    )
  ])

  assert.match(page, /@use "\.\/page";/u)

  const root = postcss().process(pageStyles, {
    from: "page.scss",
    parser: scss
  }).root
  const owners: { properties: string[]; selector: string }[] = []
  root.walkRules((rule) => {
    for (const selector of rule.selectors.filter((candidate) =>
      candidate.includes(".page-tags")
    )) {
      const properties = rule.nodes.flatMap((node) =>
        node.type === "decl" && ["display", "justify-content"].includes(node.prop)
          ? [node.prop]
          : []
      )
      if (properties.length > 0) owners.push({ selector, properties })
    }
  })

  assert.deepEqual(owners, [
    {
      selector: ".sigma-esque-container .page-tags",
      properties: ["display", "justify-content"]
    }
  ])
  assert.equal(
    owners.some(({ selector }) => selector.trim() === ".page-tags"),
    false
  )
})

test("the Wikidot shell never claims layout ownership of the imported mobile nav", async () => {
  const layout = await fs.readFile(
    new URL("../src/lib/sigma-esque/wikidot.svelte", import.meta.url),
    "utf8"
  )

  const styles = /<style\b[^>]*>(?<body>[\s\S]*?)<\/style>/u.exec(layout)?.groups?.body
  assert.ok(styles)

  const root = postcss().process(styles, { from: "wikidot.svelte", parser: scss }).root
  const offending: string[] = []

  root.walkRules((rule) => {
    if (!/\.(?:mobile-top-bar|open-menu)\b/u.test(rule.selector)) return

    for (const node of rule.nodes) {
      if (node.type !== "decl") continue
      if (
        /^(?:position|(?:inset|(?:top|right|bottom|left))|float|transform|display|margin)$/u.test(
          node.prop
        )
      ) {
        offending.push(`${rule.selector} { ${node.prop}: ${node.value} }`)
      }
    }
  })

  assert.deepEqual(offending, [])
})

test("the Wikidot header exposes the three legacy extension hooks in source order", async () => {
  const layout = await fs.readFile(
    new URL("../src/lib/sigma-esque/wikidot.svelte", import.meta.url),
    "utf8"
  )
  const header =
    /<div id="header">(?<body>[\s\S]*?)<\/div>\s*<div id="content-wrap">/u.exec(layout)
      ?.groups?.body

  assert.ok(header)
  const hookIds = [...header.matchAll(/id="(header-extra-div-[123])"/gu)].map(
    (match) => match[1]
  )
  assert.deepEqual(hookIds, [
    "header-extra-div-1",
    "header-extra-div-2",
    "header-extra-div-3"
  ])
  assert.match(header, /@render loginStatus\?\.\(\)[\s\S]*header-extra-div-1/u)
})

test("the Wikidot shell preserves the legacy two-input search chrome", async () => {
  const layout = await fs.readFile(
    new URL("../src/routes/+layout.svelte", import.meta.url),
    "utf8"
  )

  assert.match(layout, /<div id="search-top-box" class="form-search">/u)
  assert.match(
    layout,
    /<form(?=[^>]*id="search-top-box-form")(?=[^>]*action="dummy")(?=[^>]*class="input-append")[^>]*>/u
  )
  assert.match(
    layout,
    /<input(?=[^>]*id="search-top-box-input")(?=[^>]*class="text empty search-query")(?=[^>]*type="text")[^>]*>/u
  )
  assert.match(
    layout,
    /<input(?=[^>]*name="search")(?=[^>]*class="button btn")(?=[^>]*type="submit")(?=[^>]*value="Search")[^>]*>/u
  )
})

test("the Wikidot error dialog exposes the real visible display state", async () => {
  const popup = await fs.readFile(
    new URL("../src/lib/popup/error.svelte", import.meta.url),
    "utf8"
  )

  assert.match(popup, /id="odialog-container"\s+style:display="block"/u)
  assert.match(popup, /\.modal-container > \.modal \{/u)
  assert.match(popup, /\.modal-container > \.modal h2 \{/u)
  assert.doesNotMatch(popup, /^\s*\.modal\s*\{/mu)
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
