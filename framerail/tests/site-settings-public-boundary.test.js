// @ts-nocheck
import assert from "node:assert/strict"
import { once } from "node:events"
import { readFile } from "node:fs/promises"
import { createServer as createHttpServer } from "node:http"
import { fileURLToPath } from "node:url"
import vm from "node:vm"
import { after, before, describe, it } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const siteId = 17
const settingsRevision = 4
const sessionToken = "site-settings-session"

let previousWorkingDirectory
let vite
let render
let readable
let client
let originalClientRequest
let adminData
let siteSettingsComponent
let layoutSettingsComponent
let analyticsSettingsComponent
let membershipSettingsComponent
let rootLayoutComponent
let canonicalAdminPage
let legacyAdminPage
let canonicalAdminError
let legacyAdminError
let canonicalAdminServer
let legacyAdminServer
let wikidotCollapsibles

const requestContext = (
  data,
  { error = null, routeId = "/[x+2d]/admin", url = "https://wikijump.test/--/admin" } = {}
) => {
  const page = {
    data,
    error,
    form: null,
    params: {},
    route: { id: routeId },
    state: {},
    status: error ? 403 : 200,
    url: new URL(url)
  }
  const updated = readable(false)
  return new Map([
    ["__request__", { page }],
    [
      "__svelte__",
      {
        page: readable(page),
        navigating: readable(null),
        updated: { subscribe: updated.subscribe, check: async () => false }
      }
    ]
  ])
}

const renderComponent = (component, props, context = requestContext(adminData)) => {
  return render(component, { props, context })
}

const actionEvent = (action, fields, trustedSiteId = siteId) => {
  const data = new FormData()
  for (const [name, value] of Object.entries(fields)) data.set(name, String(value))
  return {
    request: new Request(`https://wikijump.test/--/admin?/${action}`, {
      method: "POST",
      body: data,
      headers: {
        "X-Wikijump-Site-Id": String(trustedSiteId),
        "X-Wikijump-Site-Slug": "test"
      }
    }),
    cookies: {
      get(name) {
        assert.equal(name, "wikijump_token")
        return sessionToken
      }
    },
    getClientAddress: () => "192.0.2.63"
  }
}

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ render } = await vite.ssrLoadModule("svelte/server"))
  ;({ readable } = await vite.ssrLoadModule("svelte/store"))
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request

  const { loadAdminPage } = await vite.ssrLoadModule("/src/lib/server/load/admin.ts")
  client.request = async (method) => {
    if (method === "admin_view") {
      return { type: "site_found", data: { categories: [], page_templates: [] } }
    }
    if (method === "membership_application_list") return []
    if (method === "translate") return {}
    throw new Error(`Unexpected Deepwell method ${method}`)
  }
  try {
    const site = {
      site_id: siteId,
      settings_revision: settingsRevision,
      slug: "test",
      name: "Test Wiki",
      tagline: "Compatibility fixture",
      description: "Site settings public-boundary fixture",
      default_page: "start",
      welcome_page: "welcome",
      locale: "en",
      layout: "wikidot",
      show_top_toolbar: false,
      show_bottom_toolbar: false,
      promote_on_other_sites: true
    }
    const request = new Request("https://wikijump.test/--/admin", {
      headers: {
        "X-Wikijump-Site-Id": String(siteId),
        "X-Wikijump-Site-Slug": "test"
      }
    })
    adminData = await loadAdminPage(request, { get: () => sessionToken }, async () => ({
      site,
      site_settings: {
        toolbars: { top: false, bottom: false, promote: true },
        membership: {
          application_enabled: false,
          password_enabled: false,
          password_configured: false
        }
      },
      promoted_sites: [],
      site_file_domain: "test.wjfiles.localhost",
      license_name: "CC BY-SA 3.0",
      license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
      license_kind: "standard",
      license_html: null,
      user_session: null,
      locales: ["en-US", "en"]
    }))
  } finally {
    client.request = originalClientRequest
  }

  ;({ default: siteSettingsComponent } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/SiteSettings.svelte"
  ))
  ;({ default: layoutSettingsComponent } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/LayoutSettings.svelte"
  ))
  ;({ default: analyticsSettingsComponent } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/AnalyticsSettings.svelte"
  ))
  ;({ default: membershipSettingsComponent } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/MembershipSettings.svelte"
  ))
  ;({ default: rootLayoutComponent } = await vite.ssrLoadModule(
    "/src/routes/+layout.svelte"
  ))
  ;({ default: canonicalAdminPage } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/+page.svelte"
  ))
  ;({ default: legacyAdminPage } = await vite.ssrLoadModule(
    "/src/routes/_admin/+page.svelte"
  ))
  ;({ default: canonicalAdminError } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/+error.svelte"
  ))
  ;({ default: legacyAdminError } = await vite.ssrLoadModule(
    "/src/routes/_admin/+error.svelte"
  ))
  ;({ wikidotCollapsibles } = await vite.ssrLoadModule(
    "/src/lib/wikidot/wikidot-collapsibles.ts"
  ))
  canonicalAdminServer = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/admin/+page.server.ts"
  )
  legacyAdminServer = await vite.ssrLoadModule("/src/routes/_admin/+page.server.ts")
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

describe("Wikidot site settings public boundaries", () => {
  it("server-renders the seven general controls in captured order and keeps layout separate", () => {
    const siteBody = renderComponent(siteSettingsComponent, { data: adminData }).body
    const controls = [
      'name="unixName"',
      'name="name"',
      'name="subtitle"',
      'id="sm-general-language"',
      'id="site-description-field"',
      'id="sm-general-start"',
      'id="sm-general-welcome"'
    ]
    let previous = -1
    for (const control of controls) {
      const index = siteBody.indexOf(control)
      assert.ok(index > previous, `${control} must follow the previous control`)
      previous = index
    }
    assert.match(siteBody, /<form id="sm-general-form"/u)
    assert.doesNotMatch(siteBody, /name="layout"/u)
    assert.match(siteBody, /<div class="accordion" id="accordion2">/u)
    assert.match(
      siteBody,
      /<a(?=[^>]*class="accordion-toggle")(?=[^>]*data-toggle="collapse")(?=[^>]*data-parent="#accordion2")(?=[^>]*href="#collapseOne")(?=[^>]*aria-expanded="false")(?=[^>]*aria-controls="collapseOne")[^>]*>/u
    )
    assert.match(siteBody, /<div id="collapseOne" class="accordion-body collapse">/u)
    assert.doesNotMatch(siteBody, /id="collapseOne"[^>]*hidden/u)
    assert.match(
      siteBody,
      /<div class="autocomplete-container">\s*<input[^>]+id="sm-general-start"/u
    )
    assert.match(
      siteBody,
      /<div class="autocomplete-container">\s*<input[^>]+id="sm-general-welcome"/u
    )
    assert.match(siteBody, /Welcome page for new members/u)

    const layoutBody = renderComponent(layoutSettingsComponent, {
      data: adminData
    }).body
    assert.match(layoutBody, /id="wikijump-layout-settings"/u)
    assert.match(layoutBody, /name="layout"/u)
  })

  it("keeps the advanced settings accordion collapsed until click or keyboard activation", () => {
    class FakeElement {
      constructor() {
        this.listeners = new Map()
        this.attributes = new Map()
        const classes = new Set()
        this.classList = {
          contains: (name) => classes.has(name),
          toggle: (name, force) => {
            const enabled = force ?? !classes.has(name)
            if (enabled) classes.add(name)
            else classes.delete(name)
            return enabled
          }
        }
      }

      addEventListener(type, listener) {
        this.listeners.set(type, listener)
      }

      removeEventListener(type) {
        this.listeners.delete(type)
      }

      contains(element) {
        return element === this.link
      }

      closest(selector) {
        return selector === 'a.accordion-toggle[data-toggle="collapse"]' ? this : null
      }

      getAttribute(name) {
        return this.attributes.get(name) ?? null
      }

      setAttribute(name, value) {
        this.attributes.set(name, String(value))
      }
    }

    const root = new FakeElement()
    const link = new FakeElement()
    const panel = new FakeElement()
    root.link = link
    root.querySelector = (selector) => (selector === "#collapseOne" ? panel : null)
    link.attributes.set("href", "#collapseOne")
    link.attributes.set("aria-controls", "collapseOne")
    link.attributes.set("aria-expanded", "false")
    panel.classList.toggle("collapse", true)

    const previousElement = globalThis.Element
    const previousHTMLElement = globalThis.HTMLElement
    globalThis.Element = FakeElement
    globalThis.HTMLElement = FakeElement
    try {
      const behavior = wikidotCollapsibles(root)
      assert.equal(panel.classList.contains("collapse"), true)
      assert.equal(panel.classList.contains("in"), false)
      assert.equal(link.getAttribute("aria-expanded"), "false")
      assert.equal(link.getAttribute("aria-controls"), "collapseOne")

      let prevented = false
      root.listeners.get("click")({
        target: link,
        preventDefault: () => {
          prevented = true
        }
      })
      assert.equal(panel.classList.contains("in"), true)
      assert.equal(link.getAttribute("aria-expanded"), "true")
      assert.equal(prevented, true)

      root.listeners.get("keydown")({
        key: "Enter",
        target: link,
        preventDefault: () => {}
      })
      assert.equal(panel.classList.contains("in"), false)
      assert.equal(link.getAttribute("aria-expanded"), "false")

      root.listeners.get("keydown")({
        key: " ",
        target: link,
        preventDefault: () => {}
      })
      assert.equal(panel.classList.contains("in"), true)
      assert.equal(link.getAttribute("aria-expanded"), "true")
      behavior.destroy()
    } finally {
      if (previousElement === undefined) delete globalThis.Element
      else globalThis.Element = previousElement
      if (previousHTMLElement === undefined) delete globalThis.HTMLElement
      else globalThis.HTMLElement = previousHTMLElement
    }
  })

  it("server-renders analytics, theme, and toolbar settings from the request view", () => {
    const enabledData = {
      site: {
        name: "Rendered settings fixture",
        slug: "rendered-settings",
        locale: "en",
        layout: "wikidot",
        from_wikidot: true,
        top_bar_page: "nav:top",
        side_bar_page: "nav:side"
      },
      site_settings: {
        google_analytics: { enabled: true, profile: "UA-1-2" },
        toolbars: { top: true, bottom: true, promote: true },
        membership: {
          application_enabled: false,
          password_enabled: false,
          password_configured: false
        }
      },
      promoted_sites: [
        {
          slug: "alpha-site",
          name: "Alpha Site",
          description: "Alpha description"
        },
        {
          slug: "beta-site",
          name: "Beta & Site",
          description: "Beta <description>"
        }
      ],
      theme: { type: "external", url: "https://cdn.scpwiki.com/site.css" },
      license_name: "CC BY-SA 3.0",
      license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
      license_kind: "standard",
      license_html: null,
      user_session: null,
      internationalization: {}
    }
    const enabled = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(enabledData, { routeId: "/[slug]/[...extra]" })
    )
    assert.match(
      enabled.head,
      /name="wikidot-site-analytics-profile" content="UA-1-2" data-wikidot-site-analytics-valid="true"/u
    )
    assert.match(
      enabled.head,
      /data-wikidot-site-theme="" href="https:\/\/cdn\.scpwiki\.com\/site\.css"/u
    )
    assert.match(enabled.body, /id="navi-bar"/u)
    assert.match(enabled.body, /id="footer-bar"/u)
    assert.match(enabled.body, />Other interesting sites<\/h2>/u)
    assert.match(
      enabled.body,
      /href="http:\/\/alpha-site\.wikidot\.com"[^>]*><img class="thumbnail" alt="" src="http:\/\/thumbnails\.wdfiles\.com\/thumbnail\/site\/alpha-site\.wikidot\.com\/80\.jpg"/u
    )
    assert.match(enabled.body, />Alpha Site<\/a>/u)
    assert.match(enabled.body, />Alpha description<\/div>/u)
    assert.match(enabled.body, />Beta &amp; Site<\/a>/u)
    assert.match(enabled.body, />Beta &lt;description><\/div>/u)
    assert.ok(
      enabled.body.indexOf('id="license-area"') < enabled.body.indexOf('id="footer-bar"'),
      "bottom toolbar must follow the license and extra chrome like Wikidot"
    )

    const previewData = { ...enabledData, theme_previewer_no_ui: true }
    const blank = renderComponent(
      rootLayoutComponent,
      {},
      requestContext({ ...previewData, theme_previewer_blank: true }, {
        routeId: "/[slug]/[...extra]"
      })
    )
    assert.match(blank.head, /wikidot-base-[a-z0-9]+\.css/u)
    assert.match(blank.head, /pagerate-[a-z0-9]+\.css/u)
    assert.doesNotMatch(blank.head, /data-wikidot-site-theme|styles\/sigma-/u)

    const preview = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(previewData, {
        routeId: "/[slug]/[...extra]",
        url: "https://wikijump.test/start?theme_url=https%3A%2F%2Fcdn.scpwiki.com%2Ftheme%2Fpreview.css"
      })
    )
    assert.match(
      preview.head,
      /data-wikidot-theme-preview="" href="https:\/\/cdn\.scpwiki\.com\/theme\/preview\.css" rel="stylesheet"/u
    )

    const rejectedPreview = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(previewData, {
        routeId: "/[slug]/[...extra]",
        url: "https://wikijump.test/start?theme_url=https%3A%2F%2Fevil.example%2Ftheme.css"
      })
    )
    assert.doesNotMatch(rejectedPreview.head, /data-wikidot-theme-preview/u)

    const unrecognizedPreview = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(enabledData, {
        routeId: "/[slug]/[...extra]",
        url: "https://wikijump.test/start?theme_url=https%3A%2F%2Fcdn.scpwiki.com%2Ftheme%2Fpreview.css"
      })
    )
    assert.doesNotMatch(unrecognizedPreview.head, /data-wikidot-theme-preview/u)

    const disabledData = structuredClone(enabledData)
    disabledData.site_settings.google_analytics = { enabled: false, profile: null }
    disabledData.site_settings.toolbars.top = false
    disabledData.site_settings.toolbars.bottom = false
    disabledData.theme = { type: "built_in", id: 1 }
    const disabled = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(disabledData, { routeId: "/[slug]/[...extra]" })
    )
    assert.doesNotMatch(disabled.head, /wikidot-site-analytics-profile/u)
    assert.doesNotMatch(disabled.head, /data-wikidot-site-theme/u)
    assert.doesNotMatch(disabled.head, /id="internal-style"/u)
    assert.doesNotMatch(disabled.body, /id="navi-bar"/u)
    assert.doesNotMatch(disabled.body, /id="footer-bar"/u)

    const variantData = structuredClone(disabledData)
    variantData.theme = { type: "built_in", id: 3 }
    const variant = renderComponent(
      rootLayoutComponent,
      {},
      requestContext(variantData, { routeId: "/[slug]/[...extra]" })
    )
    assert.match(
      variant.head,
      /<style type="text\/css" id="internal-style">\n@import url\(http:\/\/d3g0gp89917ko0\.cloudfront\.net\/v--7690939296dc\/common--theme\/clean\/css\/style\.css\);\n@import url\(http:\/\/d3g0gp89917ko0\.cloudfront\.net\/v--7690939296dc\/common--theme\/clean-no-side-bar\/css\/style\.css\);\n<\/style>/u
    )
    assert.doesNotMatch(variant.head, /common--theme\/base\/css/u)

    const analyticsBody = renderComponent(analyticsSettingsComponent, {
      data: {
        ...adminData,
        site: {
          ...adminData.site,
          google_analytics_enabled: false,
          google_analytics_profile: null
        }
      }
    }).body
    assert.match(analyticsBody, />Profile key<\/label>/u)
    assert.match(analyticsBody, />Use Google Analytics<\/label>/u)
  })

  it("rejects analytics profiles with trailing line terminators", () => {
    const data = {
      site: {
        name: "Line terminator fixture",
        slug: "line-terminator-fixture",
        locale: "en",
        layout: "wikidot"
      },
      site_settings: {
        google_analytics: { enabled: true, profile: "UA-1-2" },
        membership: {
          application_enabled: false,
          password_enabled: false,
          password_configured: false
        }
      },
      theme: { type: "built_in", id: 1 },
      license_name: "CC BY-SA 3.0",
      license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
      license_kind: "standard",
      license_html: null,
      user_session: null,
      internationalization: {}
    }
    for (const [name, terminator] of [
      ["CR", "\r"],
      ["LF", "\n"],
      ["U+2028", "\u2028"],
      ["U+2029", "\u2029"]
    ]) {
      data.site_settings.google_analytics.profile = `UA-1-2${terminator}`
      const rendered = renderComponent(
        rootLayoutComponent,
        {},
        requestContext(data, { routeId: "/[slug]/[...extra]" })
      )

      assert.doesNotMatch(rendered.head, /wikidot-site-analytics-profile/u, name)
      assert.doesNotMatch(rendered.head, /UA-1-2/u, name)
    }
  })

  it("serves analytics through nonce-protected full-document SSR without a remote loader", async () => {
    client.request = async (method) => {
      if (method === "preload_view") {
        return {
          site: {
            site_id: siteId,
            settings_revision: settingsRevision,
            slug: "test",
            name: "Test Wiki",
            tagline: "Compatibility fixture",
            description: "Site settings public-boundary fixture",
            default_page: "start",
            welcome_page: "welcome",
            locale: "en",
            layout: "wikidot",
            from_wikidot: true,
            top_bar_page: "nav:top",
            side_bar_page: "nav:side",
            favicon_source: null,
            ios_icon_source: null,
            windows_tile_source: null
          },
          site_settings: {
            google_analytics: { enabled: true, profile: "UA-1-2" },
            toolbars: { top: true, bottom: false, promote: true },
            membership: {
              application_enabled: false,
              password_enabled: false,
              password_configured: false
            }
          },
          promoted_sites: [],
          site_file_domain: "test.wjfiles.localhost",
          license_name: "CC BY-SA 3.0",
          license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
          license_kind: "standard",
          license_html: null,
          user_session: null
        }
      }
      if (method === "admin_view") {
        return { type: "site_found", data: { categories: [], page_templates: [] } }
      }
      if (method === "translate") return {}
      throw new Error(`Unexpected Deepwell method ${method}`)
    }

    const server = createHttpServer(vite.middlewares)
    server.listen(0, "127.0.0.1")
    await once(server, "listening")
    try {
      const address = server.address()
      assert.ok(address && typeof address === "object")
      const response = await fetch(`http://127.0.0.1:${address.port}/_admin`, {
        headers: {
          accept: "text/html",
          "accept-language": "en",
          "X-Wikijump-Site-Id": String(siteId),
          "X-Wikijump-Site-Slug": "test"
        }
      })
      const document = await response.text()

      assert.equal(response.status, 200)
      assert.match(response.headers.get("content-type") ?? "", /text\/html/u)
      assert.match(document, /name="wikidot-site-analytics-profile" content="UA-1-2"/u)
      assert.match(document, /userTracker\._setAccount/u)
      assert.match(document, /userTracker\._trackPageview/u)
      assert.doesNotMatch(
        document,
        /google-analytics\.com|document\.createElement|\.src\s*=/u
      )

      const contentSecurityPolicy = response.headers.get("content-security-policy") ?? ""
      const analyticsNonce = /'nonce-([^']+)'/u.exec(contentSecurityPolicy)?.[1]
      assert.ok(analyticsNonce)
      assert.notEqual(analyticsNonce, "%sveltekit.nonce%")
      const analyticsBodyIndex = document.indexOf("const analyticsProfile")
      assert.notEqual(analyticsBodyIndex, -1)
      const analyticsScriptIndex = document.lastIndexOf("<script", analyticsBodyIndex)
      assert.notEqual(analyticsScriptIndex, -1)
      const analyticsOpeningTagEnd = document.indexOf(">", analyticsScriptIndex)
      assert.notEqual(analyticsOpeningTagEnd, -1)
      assert.equal(
        document.slice(analyticsScriptIndex, analyticsOpeningTagEnd + 1),
        `<script nonce="${analyticsNonce}">`
      )
      assert.match(
        document,
        /name="wikidot-site-analytics-profile" content="UA-1-2" data-wikidot-site-analytics-valid="true"/u
      )
    } finally {
      await new Promise((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()))
      })
      client.request = originalClientRequest
    }
  })

  it("does not bootstrap analytics from an invalid unmarked meta", async () => {
    const template = await readFile(new URL("../src/app.html", import.meta.url), "utf8")
    const scriptStart = template.indexOf("      const analyticsProfile")
    const scriptEnd = template.indexOf("    </script>", scriptStart)
    assert.ok(scriptStart >= 0)
    assert.ok(scriptEnd > scriptStart)

    const selectors = []
    const context = {
      document: {
        querySelector(selector) {
          selectors.push(selector)
          return selector === 'meta[name="wikidot-site-analytics-profile"]'
            ? { content: "UA-1-2\n" }
            : null
        }
      },
      globalThis: {}
    }
    vm.runInNewContext(template.slice(scriptStart, scriptEnd), context)

    assert.deepEqual(selectors, [
      'meta[name="wikidot-site-analytics-profile"][data-wikidot-site-analytics-valid="true"]'
    ])
    assert.equal(context.globalThis._gaq, undefined)
  })

  it("routes settings writes through the trusted site and revision-bound action seam", async () => {
    const cases = [
      {
        action: "site",
        fields: {
          siteId,
          expectedSettingsRevision: settingsRevision,
          name: "Renamed wiki",
          slug: "renamed-wiki",
          tagline: "Updated tagline",
          description: "Updated description",
          defaultPage: "home",
          welcomePage: "welcome",
          locale: "en",
          action: "edit"
        },
        method: "site_update",
        expected: {
          name: "Renamed wiki",
          slug: "renamed-wiki",
          tagline: "Updated tagline",
          description: "Updated description",
          default_page: "home",
          welcome_page: "welcome",
          locale: "en"
        }
      },
      {
        action: "analytics",
        fields: {
          siteId,
          expectedSettingsRevision: settingsRevision,
          enabled: true,
          profile: "UA-1-2"
        },
        method: "site_update",
        expected: { google_analytics: { enabled: true, profile: "UA-1-2" } }
      },
      {
        action: "toolbar",
        fields: {
          siteId,
          expectedSettingsRevision: settingsRevision,
          top: true,
          bottom: false,
          promote: false
        },
        method: "site_update",
        expected: { toolbars: { top: true, bottom: false, promote: false } }
      },
      {
        action: "membership",
        fields: {
          siteId,
          expectedSettingsRevision: settingsRevision,
          applicationEnabled: true,
          passwordEnabled: true,
          password: "membership-password"
        },
        method: "site_update",
        expected: {
          membership: {
            application_enabled: true,
            password_enabled: true,
            password: "membership-password"
          }
        }
      },
      {
        action: "theme",
        fields: {
          siteId,
          categoryId: 23,
          expectedSettingsRevision: settingsRevision,
          themeType: "external",
          builtinId: 1,
          externalUrl: "https://cdn.scpwiki.com/site.css",
          customCss: ""
        },
        method: "category_update",
        expected: {
          category: 23,
          theme: { type: "external", url: "https://cdn.scpwiki.com/site.css" }
        }
      },
      {
        action: "autonumber",
        fields: {
          siteId,
          categoryId: 23,
          expectedSettingsRevision: settingsRevision,
          enabled: true
        },
        method: "category_update",
        expected: { category: 23, autonumber_enabled: true }
      },
      {
        action: "siteLayout",
        fields: {
          siteId,
          expectedSettingsRevision: settingsRevision,
          layout: "wikidot"
        },
        method: "site_update",
        expected: { layout: "wikidot" }
      }
    ]

    const calls = []
    let sessionExpired = false
    let mutationFailure = null
    client.request = async (method, params, context) => {
      calls.push({ method, params, context })
      if (method === "session_get") return sessionExpired ? null : { user_id: 41 }
      if (mutationFailure) throw mutationFailure
      return { settings_revision: settingsRevision + 1 }
    }
    try {
      for (const { action, fields, method, expected } of cases) {
        assert.equal(typeof canonicalAdminServer.actions[action], "function")
        calls.length = 0
        const result = await canonicalAdminServer.actions[action](
          actionEvent(action, fields)
        )
        assert.equal(result.form.valid, true, action)
        assert.deepEqual(calls[0], {
          method: "session_get",
          params: [sessionToken],
          context: undefined
        })
        assert.equal(calls[1].method, method, action)
        assert.equal(calls[1].params.site, siteId, action)
        assert.equal(calls[1].params.expected_settings_revision, settingsRevision, action)
        assert.equal(calls[1].params.user_id, 41, action)
        assert.equal(calls[1].params.ip_address, "192.0.2.63", action)
        assert.deepEqual(calls[1].context, { sessionToken, siteId }, action)
        for (const [name, value] of Object.entries(expected)) {
          assert.deepEqual(calls[1].params[name], value, `${action}: ${name}`)
        }
      }

      for (const [name, terminator] of [
        ["CR", "\r"],
        ["LF", "\n"],
        ["U+2028", "\u2028"],
        ["U+2029", "\u2029"]
      ]) {
        calls.length = 0
        const invalid = await canonicalAdminServer.actions.analytics(
          actionEvent("analytics", {
            siteId,
            expectedSettingsRevision: settingsRevision,
            enabled: true,
            profile: `UA-1-2${terminator}`
          })
        )
        assert.equal(invalid.status, 400, name)
        assert.deepEqual(calls, [], name)
      }

      calls.length = 0
      const denied = await canonicalAdminServer.actions.analytics(
        actionEvent("analytics", {
          siteId: 999,
          expectedSettingsRevision: settingsRevision,
          enabled: true,
          profile: "UA-1-2"
        })
      )
      assert.equal(denied.status, 403)
      assert.deepEqual(calls, [])

      sessionExpired = true
      const expired = await canonicalAdminServer.actions.analytics(
        actionEvent("analytics", {
          siteId,
          expectedSettingsRevision: settingsRevision,
          enabled: true,
          profile: "UA-1-2"
        })
      )
      assert.equal(expired.status, 401)
      assert.deepEqual(calls, [
        {
          method: "session_get",
          params: [sessionToken],
          context: undefined
        }
      ])
      sessionExpired = false

      mutationFailure = {
        message: "Site settings changed since revision 4",
        code: 4000,
        data: { expected_settings_revision: 4, current_settings_revision: 5 }
      }
      const stale = await canonicalAdminServer.actions.analytics(
        actionEvent("analytics", {
          siteId,
          expectedSettingsRevision: settingsRevision,
          enabled: true,
          profile: "UA-1-2"
        })
      )
      assert.equal(stale.status, 500)
      assert.equal(stale.data.form.valid, true)
      assert.equal(stale.data.message, "Site settings changed since revision 4")
      assert.equal(stale.data.code, 4000)
      assert.deepEqual(stale.data.data, {
        expected_settings_revision: 4,
        current_settings_revision: 5
      })
    } finally {
      client.request = originalClientRequest
    }
  })

  it("renders membership policy without exposing a configured password and binds review actions", async () => {
    const data = structuredClone(adminData)
    data.site_settings.membership = {
      application_enabled: true,
      password_enabled: true,
      password_configured: true
    }
    data.membershipApplications = [
      { user_id: 91, user_name: "Applicant", comment: "Please let me in" }
    ]
    const body = renderComponent(membershipSettingsComponent, { data }).body
    assert.match(body, /Allow membership applications/u)
    assert.match(body, /Allow membership by password/u)
    assert.match(body, /Leave blank to keep the configured password\./u)
    assert.match(body, /Membership application from Applicant/u)
    assert.match(body, /Please let me in/u)
    assert.match(body, /id="membership-password"[^>]*type="password"[^>]*value=""/u)
    assert.equal(Object.hasOwn(data.site_settings.membership, "password"), false)
    assert.equal(Object.hasOwn(data.site_settings.membership, "password_hash"), false)

    const calls = []
    client.request = async (method, params, context) => {
      calls.push({ method, params, context })
      if (method === "session_get") return { user_id: 41 }
      if (method === "membership_application_review") return "accepted"
      throw new Error(`Unexpected Deepwell method ${method}`)
    }
    try {
      const result = await canonicalAdminServer.actions.membershipReview(
        actionEvent("membershipReview", {
          siteId,
          userId: 91,
          decision: "accept",
          reply: "welcome"
        })
      )
      assert.equal(result.form.valid, true)
      assert.deepEqual(calls, [
        { method: "session_get", params: [sessionToken], context: undefined },
        {
          method: "membership_application_review",
          params: {
            site_id: siteId,
            user_id: 91,
            decision: "accept",
            reply: "welcome",
            ip_address: "192.0.2.63"
          },
          context: { sessionToken, siteId }
        }
      ])
    } finally {
      client.request = originalClientRequest
    }
  })

  it("page template settings bind the category mutation to the trusted site actor", async () => {
    const calls = []
    client.request = async (method, params, context) => {
      calls.push({ method, params, context })
      if (method === "session_get") return { user_id: 41 }
      if (method === "category_update") return { category_id: 23, template_page_id: 91 }
      throw new Error(`Unexpected Deepwell method ${method}`)
    }

    const result = await canonicalAdminServer.actions.template(
      actionEvent("template", {
        siteId,
        categoryId: 23,
        templatePageId: 91
      })
    )

    assert.equal(result.form.valid, true)
    assert.deepEqual(calls, [
      {
        method: "session_get",
        params: [sessionToken],
        context: undefined
      },
      {
        method: "category_update",
        params: {
          site: siteId,
          category: 23,
          user_id: 41,
          template_page_id: 91,
          ip_address: "192.0.2.63"
        },
        context: { sessionToken, siteId }
      }
    ])
    assert.equal(
      legacyAdminServer.actions.template,
      canonicalAdminServer.actions.template
    )
  })

  it("serves the legacy admin route through the same rendered page and action seam", () => {
    assert.equal(legacyAdminServer.load, canonicalAdminServer.load)
    assert.equal(legacyAdminServer.actions, canonicalAdminServer.actions)

    const canonicalBody = renderComponent(canonicalAdminPage, {
      data: adminData
    }).body
    const legacyBody = renderComponent(legacyAdminPage, {
      data: adminData,
      form: null,
      params: {}
    }).body
    const formActions = (body) =>
      [...body.matchAll(/<form[^>]+action="([^"]+)"/gu)].map((match) => match[1])
    assert.deepEqual(formActions(legacyBody), formActions(canonicalBody))
    for (const id of ["sm-general-form", "wikijump-layout-settings"]) {
      assert.match(legacyBody, new RegExp(`id="${id}"`, "u"))
    }

    const error = {
      view: "admin_permissions",
      html: "<strong>Permission boundary fixture</strong>",
      internationalization: {
        error: "Error",
        "admin-unauthorized":
          "You do not have the permissions to access the admin panel.",
        "error-api.GENERIC": "Something went wrong with your request."
      }
    }
    const errorContext = requestContext(adminData, {
      error,
      routeId: "/_admin"
    })
    const canonicalErrorBody = renderComponent(canonicalAdminError, {}, errorContext).body
    const legacyErrorBody = renderComponent(
      legacyAdminError,
      {},
      requestContext(adminData, { error, routeId: "/_admin" })
    ).body
    for (const body of [canonicalErrorBody, legacyErrorBody]) {
      assert.doesNotMatch(body, /UNTRANSLATED:/u)
      assert.match(body, /<h1[^>]*>Error<\/h1>/u)
      assert.match(body, /You do not have the permissions to access the admin panel\./u)
      assert.match(body, /<strong>Permission boundary fixture<\/strong>/u)
    }
  })
})
