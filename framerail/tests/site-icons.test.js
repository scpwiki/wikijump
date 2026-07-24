import assert from "node:assert/strict"
import test from "node:test"

import {
  FAVICON_ROUTE_PREFIX,
  IOS_ICON_DECLARATIONS,
  IOS_ICON_ROUTE_PREFIX,
  faviconDeclaration,
  hasIosIcons
} from "../src/lib/site-icons.ts"

test("favicon declaration keeps Wikidot's local route rather than the configured source", () => {
  // Live scp-wiki declares /local--favicon/favicon.gif with type image/gif.
  assert.deepEqual(
    faviconDeclaration({
      favicon_source: "https://scp-wiki.wdfiles.com/local--files/site/favicon.gif"
    }),
    { href: `${FAVICON_ROUTE_PREFIX}favicon.gif`, type: "image/gif" }
  )
})

test("favicon declaration carries the type matching the configured extension", () => {
  assert.deepEqual(faviconDeclaration({ favicon_source: "icon.png" }), {
    href: `${FAVICON_ROUTE_PREFIX}favicon.png`,
    type: "image/png"
  })
  assert.deepEqual(faviconDeclaration({ favicon_source: "icon.ICO" }), {
    href: `${FAVICON_ROUTE_PREFIX}favicon.ico`,
    type: "image/x-icon"
  })
})

test("a site without a usable icon declares nothing", () => {
  assert.equal(faviconDeclaration(null), null)
  assert.equal(faviconDeclaration({ favicon_source: null }), null)
  assert.equal(faviconDeclaration({ favicon_source: "" }), null)
  assert.equal(
    faviconDeclaration({ favicon_source: "icon" }),
    null,
    "an extensionless source has no type to declare"
  )
  assert.equal(
    faviconDeclaration({ favicon_source: "icon.webp" }),
    null,
    "an unmapped extension must not guess a MIME type"
  )
})

test("query strings and fragments do not become part of the extension", () => {
  assert.deepEqual(faviconDeclaration({ favicon_source: "icon.gif?v=2" }), {
    href: `${FAVICON_ROUTE_PREFIX}favicon.gif`,
    type: "image/gif"
  })
})

test("iOS touch icons reproduce the three filenames and sizes Wikidot declares", () => {
  assert.equal(hasIosIcons({ ios_icon_source: "iosicon.png" }), true)
  assert.equal(hasIosIcons({ ios_icon_source: null }), false)
  assert.equal(hasIosIcons(null), false)

  assert.deepEqual(
    IOS_ICON_DECLARATIONS.map(
      (icon) => `${IOS_ICON_ROUTE_PREFIX}${icon.filename} ${icon.sizes ?? "-"}`
    ),
    [
      "/local--iosicon/iosicon_57.png -",
      "/local--iosicon/iosicon_72.png 72x72",
      "/local--iosicon/iosicon.png 114x114"
    ]
  )
})
