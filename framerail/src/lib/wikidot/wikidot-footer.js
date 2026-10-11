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
 * `Report a bug` and `Flag as objectionable` are inert anchors on Wikidot
 * too. The click runs `WIKIDOT.page.listeners.pageBugReport` /
 * `flagPageObjectionable`, which request the in-page
 * `report/BugReportModule` and `report/FlagPageModule` Ajax modules
 * instead of navigating. Reproducing the current site's root `/` for them
 * sent readers to the wrong site, which is the placeholder behaviour this
 * replaces.
 */
export const WIKIDOT_FOOTER_LINKS = Object.freeze([
  { label: "Help", href: "https://www.wikidot.com/doc" },
  {
    label: "Terms of Service",
    href: "https://www.wikidot.com/legal:terms-of-service"
  },
  { label: "Privacy", href: "https://www.wikidot.com/legal:privacy-policy" },
  { label: "Report a bug", href: "javascript:;" },
  { label: "Flag as objectionable", href: "javascript:;" }
])

/**
 * Japanese Wikidot footer destinations. The live SCP-JP footer captured
 * 2026-09-29 emits the same targets with the Japanese labels.
 */
export const WIKIDOT_FOOTER_LINKS_JA = Object.freeze([
  { label: "ヘルプ", href: "https://www.wikidot.com/doc" },
  { label: "利用規約", href: "https://www.wikidot.com/legal:terms-of-service" },
  { label: "プライバシー", href: "https://www.wikidot.com/legal:privacy-policy" },
  { label: "バグを報告", href: "javascript:;" },
  { label: "不快フラグを立てる", href: "javascript:;" }
])

export const WIKIDOT_POWERED_BY = "Powered by Wikidot.com"

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

    return `特に指定がない限り、このサイトのすべてのコンテンツは<a href="${url}">${name}</a> の元で利用可能です。`
  }

  return `Unless otherwise stated, the content of this page is licensed under <a href="${url}">${name}</a>`
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
