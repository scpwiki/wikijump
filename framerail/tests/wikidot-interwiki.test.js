import { strict as assert } from "node:assert"
import test from "node:test"
import vm from "node:vm"

import {
  buildWikidotInterwikiFrameHtml,
  extractWikidotInterwikiLinks
} from "../src/lib/wikidot/wikidot-interwiki.js"
import {
  buildWikidotStyleFrameHtml,
  extractWikidotStyleFrameStylesheets,
  isUsableStyleFrameCss,
  localizeWikidotThemeUrl
} from "../src/lib/wikidot/wikidot-styleframe.js"

test("extracts priority-ordered styleFrame stylesheets for initial document CSS", () => {
  assert.deepEqual(
    extractWikidotStyleFrameStylesheets(
      [
        '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=2&amp;theme=https%3A%2F%2Fcdn.scpwiki.com%2Ftheme%2Fen%2Fbasalt%2Fbasalt-bedrock-min.css&amp;css=%7B%24css%7D"></iframe>',
        '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1&theme=https%3A%2F%2Fscp-wiki.wdfiles.com%2Flocal--code%2Ftheme%253Abasalt%2F1"></iframe>'
      ],
      "https://scp-wiki.wikijump.localhost"
    ),
    [
      {
        href: "https://scp-wiki.wjfiles.localhost/local--code/theme%3Abasalt/1",
        priority: "1",
        priorityValue: 1,
        order: 1
      },
      {
        href: "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css",
        priority: "2",
        priorityValue: 2,
        order: 0
      }
    ]
  )
})

const cromPage = {
  translations: [
    { url: "http://scp-wiki-cn.wikidot.com/1231-warning" },
    { url: "https://fondationscp.wikidot.com/1231-warning" },
    { url: "https://scp-jp.wikidot.com/1231-warning" },
    { url: "https://scpko.wikidot.com/1231-warning" },
    { url: "https://scpfoundation.net/1231-warning" },
    { url: "https://scp-vn.wikidot.com/1231-warning" }
  ],
  translationOf: null
}

const executeStyleFrame = (
  html,
  head,
  scheduledCallbacks = [],
  { frameElement = null, parentWindow = null } = {}
) => {
  const script = html.match(/<script>([\s\S]*?)<\/script>/u)?.[1]
  assert.ok(script)

  const document = {
    baseURI: "https://scp-wiki.wikijump.localhost/",
    head,
    documentElement: head,
    defaultView: {},
    querySelectorAll: (selector) => head.querySelectorAll(selector),
    createElement: (tagName) => ({
      dataset: {},
      tagName: tagName.toUpperCase(),
      remove() {
        head.removeChild(this)
      }
    })
  }
  const listeners = new Map()
  const window = {
    addEventListener: (type, callback) => listeners.set(type, callback),
    document,
    frameElement
  }
  if (parentWindow) parentWindow.document = document
  window.parent = parentWindow ?? window
  vm.runInNewContext(script, {
    document,
    setTimeout: (callback) => scheduledCallbacks.push(callback),
    URL,
    window
  })
  return { listeners, window }
}

const createHead = (...initialNodes) => {
  const children = [...initialNodes]
  let moveCount = 0
  const moveBefore = (node, referenceNode = null) => {
    moveCount += 1
    const previousIndex = children.indexOf(node)
    if (previousIndex !== -1) children.splice(previousIndex, 1)
    const referenceIndex = referenceNode === null ? -1 : children.indexOf(referenceNode)
    if (referenceNode !== null) assert.notEqual(referenceIndex, -1)
    children.splice(referenceIndex === -1 ? children.length : referenceIndex, 0, node)
  }

  return {
    appendChild: (node) => moveBefore(node),
    children,
    get moveCount() {
      return moveCount
    },
    insertBefore: moveBefore,
    removeChild: (node) => {
      const index = children.indexOf(node)
      if (index !== -1) children.splice(index, 1)
    },
    querySelectorAll: (selector) => {
      if (selector === '[data-wikidot-style-frame="wikidot-style-frame"]') {
        return children.filter(
          (node) => node.dataset.wikidotStyleFrame === "wikidot-style-frame"
        )
      }
      if (selector === "[data-wikijump-generated-css]") {
        return children.filter((node) => node.dataset.wikijumpGeneratedCss !== undefined)
      }
      if (selector === "link[data-wikidot-style-preloaded]") {
        return children.filter(
          (node) =>
            node.tagName === "LINK" && node.dataset.wikidotStylePreloaded !== undefined
        )
      }
      const owner = selector.match(/^\[data-wikidot-style-owner="(.+)"\]$/u)?.[1]
      if (owner) {
        return children.filter((node) => node.dataset.wikidotStyleOwner === owner)
      }
      assert.fail(`Unexpected selector: ${selector}`)
    }
  }
}

test("builds SCP interwiki language links from Crom translations", () => {
  assert.deepEqual(
    extractWikidotInterwikiLinks({
      community: "scp",
      lang: "en",
      sourcePath: "1231-warning",
      page: cromPage
    }).map((link) => link.label),
    ["中文", "Français", "日本語", "한국어", "Русский", "Tiếng Việt"]
  )
})

test("replaces styles owned by a reused styleFrame across page navigation", () => {
  const baseStyle = { dataset: {}, id: "base" }
  const generatedPageStyle = {
    dataset: { wikijumpGeneratedCss: "0" },
    id: "generated-page"
  }
  const head = createHead(baseStyle, generatedPageStyle)
  const parentWindow = {}
  const frameElement = { dataset: {}, isConnected: true }
  const firstCallbacks = []

  const first = executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "1",
      themes: ["https://example.com/page-a.css"]
    }),
    head,
    firstCallbacks,
    { frameElement, parentWindow }
  )
  const firstOwner = frameElement.dataset.wikidotStyleOwner
  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "2",
      themes: ["https://example.com/page-b.css"]
    }),
    head,
    [],
    { frameElement, parentWindow }
  )

  assert.notEqual(frameElement.dataset.wikidotStyleOwner, firstOwner)
  assert.deepEqual(
    head.children.map((node) => node.href ?? node.id),
    ["base", "https://example.com/page-b.css", "generated-page"]
  )

  const moveCount = head.moveCount
  firstCallbacks.forEach((callback) => callback())
  assert.equal(head.moveCount, moveCount)

  first.listeners.get("pagehide")?.()
  assert.deepEqual(
    head.children.map((node) => node.href ?? node.id),
    ["base", "https://example.com/page-b.css", "generated-page"]
  )

  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "1",
      themes: ["https://example.com/page-a.css"]
    }),
    head,
    [],
    { frameElement, parentWindow }
  )
  assert.deepEqual(
    head.children.map((node) => node.href ?? node.id),
    ["base", "https://example.com/page-a.css", "generated-page"]
  )
})

test("removes only the styles owned by the unloaded styleFrame", () => {
  const baseStyle = { dataset: {}, id: "base" }
  const head = createHead(baseStyle)
  const parentWindow = {}
  const firstFrame = { dataset: {}, isConnected: true }
  const secondFrame = { dataset: {}, isConnected: true }
  const first = executeStyleFrame(
    buildWikidotStyleFrameHtml({ themes: ["https://example.com/a.css"] }),
    head,
    [],
    { frameElement: firstFrame, parentWindow }
  )
  executeStyleFrame(
    buildWikidotStyleFrameHtml({ themes: ["https://example.com/b.css"] }),
    head,
    [],
    { frameElement: secondFrame, parentWindow }
  )

  first.listeners.get("pagehide")?.()

  assert.deepEqual(
    head.children.map((node) => node.href ?? node.id),
    ["base", "https://example.com/b.css"]
  )
  assert.equal(firstFrame.dataset.wikidotStyleOwner, undefined)
  assert.ok(secondFrame.dataset.wikidotStyleOwner)
})

test("renders Wikidot-compatible interwiki visible text for translated SCP pages", () => {
  const html = buildWikidotInterwikiFrameHtml({
    community: "scp",
    lang: "en",
    pagename: "1231-warning",
    page: cromPage
  })

  assert.match(html, /In other languages/)
  assert.doesNotMatch(html, /IN OTHER LANGUAGES/)
  assert.match(html, /中文<\/a><\/div> <div class="menu-item" name="fr"/)
  assert.match(html, /中文/)
  assert.match(html, /Français/)
  assert.match(html, /日本語/)
  assert.match(html, /한국어/)
  assert.match(html, /Русский/)
  assert.match(html, /Tiếng Việt/)
  assert.doesNotMatch(html, /English/)
})

test("builds styleFrame parent injection for theme stylesheets", () => {
  const html = buildWikidotStyleFrameHtml({
    priority: "2",
    themes: [
      "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css",
      "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1"
    ],
    css: "{$css}",
    origin: "https://scp-wiki.wikijump.localhost"
  })

  assert.match(html, /wikidot-style-theme-count" content="2"/)
  assert.match(html, /targetWindow\.document/)
  assert.match(html, /head\.insertBefore\(element, laterStyle\)/)
  assert.match(html, /restoreStyleFrameOrder/)
  assert.match(html, /link\[data-wikidot-style-preloaded\]/)
  assert.match(html, /generatedCssNodes/)
  assert.match(html, /desiredTail\.forEach/)
  assert.match(html, /if \(alreadyOrdered\) return/)
  assert.match(html, /cdn\.scpwiki\.com\/theme\/en\/basalt\/basalt-bedrock-min\.css/)
  assert.match(html, /scp-wiki\.wjfiles\.localhost\/local--code\/theme%3Abasalt\/1/)
  assert.doesNotMatch(html, /<style>\{\$css\}<\/style>/)
  assert.doesNotMatch(html, /<style>\$css<\/style>/)
})

test("adopts an SSR stylesheet instead of loading the styleFrame theme twice", () => {
  const preloaded = {
    dataset: { wikidotStylePreloaded: "", wikidotStylePriority: "2" },
    href: "https://example.com/theme.css",
    tagName: "LINK"
  }
  const head = createHead(preloaded)

  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "2",
      themes: ["https://example.com/theme.css"]
    }),
    head
  )

  assert.equal(head.children.length, 1)
  assert.equal(head.children[0], preloaded)
  assert.equal(preloaded.dataset.wikidotStylePreloaded, undefined)
  assert.equal(preloaded.dataset.wikidotStyleFrame, "wikidot-style-frame")
  assert.match(preloaded.dataset.wikidotStyleOwner, /^wikidot-style-frame-/u)
})

test("keeps app styles before priority-ordered styleFrame and generated CSS", () => {
  const baseStyle = { dataset: {}, id: "base" }
  const generatedPageStyle0 = {
    dataset: { wikijumpGeneratedCss: "0" },
    id: "generated-page-0"
  }
  const generatedPageStyle1 = {
    dataset: { wikijumpGeneratedCss: "1" },
    id: "generated-page-1"
  }
  const appStyle = { dataset: {}, id: "app" }
  const head = createHead(baseStyle, generatedPageStyle0, generatedPageStyle1, appStyle)
  const scheduledCallbacks = []

  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "2",
      themes: ["https://example.com/late.css"]
    }),
    head,
    scheduledCallbacks
  )
  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      priority: "1",
      themes: ["https://example.com/early.css"],
      css: ".included { display: none; }"
    }),
    head,
    scheduledCallbacks
  )

  assert.deepEqual(
    head.children.map((node) =>
      node.dataset.wikidotStyleFrame
        ? `${node.dataset.wikidotStylePriority}:${node.dataset.wikidotStyleId}`
        : node.id
    ),
    [
      "base",
      "app",
      "1:theme-0",
      "1:inline-css",
      "2:theme-0",
      "generated-page-0",
      "generated-page-1"
    ]
  )

  const nodeCount = head.children.length
  const moveCount = head.moveCount
  scheduledCallbacks.forEach((callback) => callback())
  assert.equal(head.children.length, nodeCount)
  assert.equal(head.moveCount, moveCount)
  assert.deepEqual(
    head.children.map((node) => node.id ?? node.dataset.wikidotStylePriority),
    ["base", "app", "1", "1", "2", "generated-page-0", "generated-page-1"]
  )
})

test("appends styleFrame CSS when there is no generated page CSS", () => {
  const baseStyle = { dataset: {}, id: "base" }
  const head = createHead(baseStyle)

  executeStyleFrame(
    buildWikidotStyleFrameHtml({
      themes: ["https://example.com/theme.css"]
    }),
    head
  )

  assert.equal(head.children[0], baseStyle)
  assert.equal(head.children[1].href, "https://example.com/theme.css")
})

test("keeps non-placeholder styleFrame inline CSS safe", () => {
  const html = buildWikidotStyleFrameHtml({
    css: "body::before { content: '</style>'; }"
  })

  assert.equal(isUsableStyleFrameCss("{$css}"), false)
  assert.equal(isUsableStyleFrameCss("$css"), false)
  assert.equal(isUsableStyleFrameCss(" body { color: red } "), true)
  assert.match(html, /<style>body::before \{ content: '<\\\/style>'; \}<\/style>/)
  assert.match(html, /const css = "body::before \{ content: '\\u003c/)
  assert.doesNotMatch(html, /<\/script>.*<\/script>/s)
})

test("localizes Wikidot local file and code theme URLs to the local file host", () => {
  assert.equal(
    localizeWikidotThemeUrl(
      "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://scp-wiki.wjfiles.localhost/local--code/theme%3Abasalt/1"
  )
  assert.equal(
    localizeWikidotThemeUrl(
      "https://scp-wiki.wdfiles.com/local--code/theme:basalt/1",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://scp-wiki.wjfiles.localhost/local--code/theme:basalt/1"
  )
  assert.equal(
    localizeWikidotThemeUrl(
      "https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css"
  )
})
