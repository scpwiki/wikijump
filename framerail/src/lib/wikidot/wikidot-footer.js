import { showWikidotReadonlyDialog } from "./wikidot-readonly-dialog.js"
import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

/**
 * Wikidot-compatible footer destinations.
 *
 * These are platform-global links: they point at Wikidot's own host and
 * never at the wiki being viewed, so they must not be fitted to any
 * captured page. The frozen live Wikidot footer for an anonymous page on
 * `sandbox-for-codex` (retained capture, 2026-09-15) and the current live
 * DOM for both SCP-Wiki and SCP-JP (`scp-173`, captured 2026-09-29) emit
 * exactly these targets, in this order:
 *
 * Help https://www.wikidot.com/doc Terms of Service
 * https://www.wikidot.com/legal:terms-of-service Privacy
 * https://www.wikidot.com/legal:privacy-policy Report a bug javascript:;
 * Flag as objectionable javascript:;
 *
 * Each anchor carries Wikidot's control id, which its hovertips attach to.
 * `Report a bug` and `Flag as objectionable` are inert anchors on Wikidot
 * too. The click runs `WIKIDOT.page.listeners.pageBugReport` /
 * `flagPageObjectionable`, which request the in-page
 * `report/BugReportModule` and `report/FlagPageModule` Ajax modules
 * instead of navigating. Reproducing the current site's root `/` for them
 * sent readers to the wrong site, which is the placeholder behaviour this
 * replaces.
 */
export const WIKIDOT_FOOTER_LINKS = Object.freeze([
  { label: "Help", href: "https://www.wikidot.com/doc", id: "wikidot-help-button" },
  {
    label: "Terms of Service",
    href: "https://www.wikidot.com/legal:terms-of-service",
    id: "wikidot-tos-button"
  },
  {
    label: "Privacy",
    href: "https://www.wikidot.com/legal:privacy-policy",
    id: "wikidot-privacy-button"
  },
  {
    label: "Report a bug",
    href: "javascript:;",
    id: "bug-report-button",
    onclick: "WIKIDOT.page.listeners.pageBugReport(event)"
  },
  {
    label: "Flag as objectionable",
    href: "javascript:;",
    id: "abuse-report-button",
    onclick: "WIKIDOT.page.listeners.flagPageObjectionable(event)"
  }
])

/**
 * Japanese Wikidot footer destinations. The live SCP-JP footer captured
 * 2026-09-29 emits the same targets with the Japanese labels.
 */
export const WIKIDOT_FOOTER_LINKS_JA = Object.freeze([
  { label: "ヘルプ", href: "https://www.wikidot.com/doc", id: "wikidot-help-button" },
  {
    label: "利用規約",
    href: "https://www.wikidot.com/legal:terms-of-service",
    id: "wikidot-tos-button"
  },
  {
    label: "プライバシー",
    href: "https://www.wikidot.com/legal:privacy-policy",
    id: "wikidot-privacy-button"
  },
  {
    label: "バグを報告",
    href: "javascript:;",
    id: "bug-report-button",
    onclick: "WIKIDOT.page.listeners.pageBugReport(event)"
  },
  {
    label: "不快フラグを立てる",
    href: "javascript:;",
    id: "abuse-report-button",
    onclick: "WIKIDOT.page.listeners.flagPageObjectionable(event)"
  }
])

export const WIKIDOT_POWERED_BY = "Powered by Wikidot.com"

/**
 * Inline style the standard Wikidot server template puts on `#footer` and
 * its `div.options`.
 */
export const WIKIDOT_FOOTER_STYLE = "display: block; visibility: visible;"

/**
 * The body of the standard Wikidot `#footer`, as the server template emits
 * it: the same text nodes, `&nbsp;|` separators, the hidden cookie-consent
 * entry and the linked "Powered by" line. The retained EN and JP shells
 * (`scp-173`, 2026-09-29) and SCP-9506 (2026-10-12) are byte-identical
 * apart from the localized link labels. The template's inline script,
 * which toggles the consent entry for the nitroAds advertising stack, is
 * third-party ad code and is not reproduced; the consent entry stays
 * hidden, as it is whenever that stack is absent.
 *
 * @param {string | null | undefined} locale
 */
export const buildWikidotFooterHtml = (locale) => {
  const links = buildWikidotFooterLinks(locale)
    .map(
      ({ label, href, id, onclick }) =>
        `<a href="${href}" id="${id}"${onclick ? ` onclick="${onclick}"` : ""}>${label}</a>`
    )
    .join("\n    &nbsp;|\n    ")
  return (
    `\n                <div class="options" style="${WIKIDOT_FOOTER_STYLE}">\n    ${links}` +
    `\n    <span id="consent-box" style="display:none">\n        &nbsp;|` +
    `\n        <a href="javascript:;" onclick="window.__cmp('showModal');">Update cookie settings</a>` +
    `\n    </span>\n\n    \n</div>` +
    `\nPowered by <a href="https://www.wikidot.com">Wikidot.com</a>\n            `
  )
}

/**
 * The exact inline handlers of the standard footer controls. They stay in
 * the served DOM for parity; CSP-safe bindings render evidenced read-only
 * modules. Unsupported actor/locale paths and the consent dialog stay
 * inert.
 */
export const WIKIDOT_FOOTER_ONCLICKS = Object.freeze([
  "WIKIDOT.page.listeners.pageBugReport(event)",
  "WIKIDOT.page.listeners.flagPageObjectionable(event)",
  "window.__cmp('showModal');"
])

/**
 * Bind the standard footer's inline handlers without compiling them:
 * assigning `onclick` replaces the attribute's handler, and the attribute
 * is only ever read as text.
 *
 * @param {HTMLElement} root
 */
export const wikidotFooterControls = (root, parameters = {}) => {
  /** @type {HTMLElement[]} */
  const bound = []
  for (const element of /** @type {NodeListOf<HTMLElement>} */ (
    root.querySelectorAll("a[onclick]")
  )) {
    if (!WIKIDOT_FOOTER_ONCLICKS.includes(element.getAttribute("onclick") ?? "")) continue
    element.onclick = () => {
      const handler = element.getAttribute("onclick")
      if (handler === WIKIDOT_FOOTER_ONCLICKS[0]) {
        showWikidotReadonlyDialog(
          root.ownerDocument,
          buildWikidotBugReportDialogHtml(parameters.locale)
        )
      } else if (
        handler === WIKIDOT_FOOTER_ONCLICKS[1] &&
        parameters.anonymous === true &&
        !isJapaneseWikidotLocale(parameters.locale)
      ) {
        showWikidotReadonlyDialog(root.ownerDocument, WIKIDOT_ANONYMOUS_FLAG_DIALOG, true)
      }
      return false
    }
    bound.push(element)
  }
  return {
    destroy() {
      for (const element of bound) element.onclick = null
    }
  }
}

// Standard public BugReportModule bodies. No report submission is made.
const BUG_REPORT_DIALOG = Object.freeze({
  "en": '<div class="title">Bug report</div>\n<div class="content">\n\t<h1>Report a problem with Wikidot.com</h1>\n\t\n\t<p>\n\t\tIf you think you have found a bug or simply something does not work as expected,       please fill a bug report.             <br/><br/>             Currently the list of bugs is maintained at:\t</p>\n\t<p style="font-weight: bold; font-size: 180%; text-align: center">\n\t\t<a href="https://feedback.wikidot.com/">feedback.wikidot.com</a>\n\t</p>\n</div>\n<div class="button-bar">\n\t<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">Close window</a> \n</div>',
  "ja": '<div class="title">バグ報告</div>\n<div class="content">\n\t<h1>Wikidot.comに問題を報告</h1>\n\t\n\t<p>\n\t\tあなたがバグを発見したり、単に何かが期待どおりに動かない場合は、バグレポートを記入してください。<br/><br/>現在、バグリストは次の場所で管理されています:\t</p>\n\t<p style="font-weight: bold; font-size: 180%; text-align: center">\n\t\t<a href="https://feedback.wikidot.com/">feedback.wikidot.com</a>\n\t</p>\n</div>\n<div class="button-bar">\n\t<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">ウィンドウを終了</a> \n</div>'
})
export const buildWikidotBugReportDialogHtml = (locale) =>
  BUG_REPORT_DIALOG[isJapaneseWikidotLocale(locale) ? "ja" : "en"]

// The observed anonymous English FlagPageModule denial. Authenticated
// reporting and untranslated dialog chrome remain unsupported.
export const WIKIDOT_ANONYMOUS_FLAG_DIALOG =
  '<div class="title modal-header">Permission error</div><div class="content modal-body"><h1>Please note:</h1><p>Please create a Wikidot account and/or sign in first</p></div><div class="button-bar modal-footer"><a href="javascript:;" class="btn btn-default button button-close-message">close message</a></div>'

export const WIKIDOT_LOGIN_LABELS = Object.freeze({
  createAccount: "Create account",
  or: "or",
  signIn: "Sign in"
})

export const WIKIDOT_LOGIN_LABELS_JA = Object.freeze({
  createAccount: "アカウントを作成",
  or: "または",
  signIn: "サインイン"
})

export const WIKIDOT_ACCOUNT_LABELS = Object.freeze({
  myAccount: "My account",
  settings: "Settings",
  signOut: "Sign out"
})

export const WIKIDOT_ACCOUNT_LABELS_JA = Object.freeze({
  myAccount: "マイアカウント",
  settings: "設定",
  signOut: "サインアウト"
})

/** @param {string | null | undefined} locale */
export const buildWikidotFooterLinks = (locale) => {
  return isJapaneseWikidotLocale(locale) ? WIKIDOT_FOOTER_LINKS_JA : WIKIDOT_FOOTER_LINKS
}

/** @param {string | null | undefined} locale */
export const buildWikidotLoginLabels = (locale) => {
  return isJapaneseWikidotLocale(locale) ? WIKIDOT_LOGIN_LABELS_JA : WIKIDOT_LOGIN_LABELS
}

/** @param {string | null | undefined} locale */
export const buildWikidotAccountLabels = (locale) => {
  return isJapaneseWikidotLocale(locale)
    ? WIKIDOT_ACCOUNT_LABELS_JA
    : WIKIDOT_ACCOUNT_LABELS
}

/**
 * @param {string | null | undefined} value
 * @returns {string}
 */
const escapeHtml = (value) => {
  return `${value ?? ""}`
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
}

/**
 * @param {string | null | undefined} licenseName
 * @returns {string}
 */
export const formatWikidotLicenseName = (licenseName, locale = "en") => {
  const name = `${licenseName ?? ""}`.replace(/\.$/, "").trim()

  if (!name) return "License"

  if (isJapaneseWikidotLocale(locale)) {
    if (/^creative commons attribution-sharealike 3\.0$/i.test(name)) {
      return "クリエイティブ・コモンズ 表示 - 継承3.0ライセンス"
    }
  }

  return /license$/i.test(name) ? name : `${name} License`
}

/**
 * @param {{
 *   licenseName?: string | null
 *   licenseUrl?: string | null
 *   licenseKind?: string | null
 *   licenseHtml?: string | null
 *   locale?: string | null
 *   sourceSite?: string | null
 * }} input
 * @returns {string}
 */
export const buildWikidotLicenseHtml = ({
  licenseName,
  licenseUrl,
  licenseKind = "standard",
  licenseHtml = null,
  locale = "en",
  sourceSite = null
}) => {
  if (licenseKind === "copyright") return ""
  if (licenseKind === "other") return licenseHtml ?? ""

  const effectiveLocale = locale ?? "en"
  const name = escapeHtml(formatWikidotLicenseName(licenseName, effectiveLocale))
  const englishName = escapeHtml(formatWikidotLicenseName(licenseName, "en"))
  const url = escapeHtml(licenseUrl || "/")

  if (isJapaneseWikidotLocale(effectiveLocale)) {
    if (sourceSite && sourceSite !== "scp-jp") {
      return `特に明記しない限り、このページのコンテンツは次のライセンスの下にあります: <a href="${url}">${englishName}</a>`
    }

    const jaUrl = escapeHtml(wikidotLicenseUrl(licenseUrl, "ja"))
    return `\n                    \n\n特に指定がない限り、このサイトのすべてのコンテンツは<a rel="license" href="${jaUrl}">${name}</a> の元で利用可能です。\n                `
  }

  const enUrl = escapeHtml(wikidotLicenseUrl(licenseUrl, "en"))
  return `\n                    Unless otherwise stated, the content of this page is licensed under\n\n <a rel="license" href="${enUrl}">${name}</a>\n                `
}

/**
 * Wikidot links a Creative Commons license to its `http://` deed, and the
 * Japanese template to the Japanese deed: `http://creativecommons.org/
 * licenses/by-sa/3.0/` on scp-wiki and `…/by-sa/3.0/deed.ja` on scp-jp
 * (retained shells, 2026-09-29). Other license URLs are kept.
 *
 * @param {string | null | undefined} licenseUrl
 * @param {"en" | "ja"} language
 */
const wikidotLicenseUrl = (licenseUrl, language) => {
  const url = licenseUrl || "/"
  if (!/^https?:\/\/creativecommons\.org\/licenses\/[^?#]*\/$/u.test(url)) return url
  const deed = url.replace(/^https:/u, "http:")
  return language === "ja" ? `${deed}deed.ja` : deed
}

/**
 * @param {boolean} importedWikidotLayout
 * @param {string | null | undefined} licenseKind
 */
export const shouldUseWikidotLicenseHtml = (importedWikidotLayout, licenseKind) =>
  importedWikidotLayout || licenseKind === "other" || licenseKind === "copyright"

/**
 * @param {{
 *       site?: { from_wikidot?: boolean | null } | null
 *       page?: { from_wikidot?: boolean | null } | null
 *       page_revision?: { from_wikidot?: boolean | null } | null
 *     }
 *   | null
 *   | undefined} data
 * @returns {boolean}
 */
export const isImportedWikidotView = (data) => {
  return !!(
    data?.site?.from_wikidot ||
    data?.page?.from_wikidot ||
    data?.page_revision?.from_wikidot
  )
}
