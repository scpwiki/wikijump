import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

export const WIKIDOT_FOOTER_LINKS = Object.freeze([
  { label: "Help", href: "/" },
  { label: "Terms of Service", href: "/" },
  { label: "Privacy", href: "/" },
  { label: "Report a bug", href: "/" },
  { label: "Flag as objectionable", href: "/" }
])

export const WIKIDOT_FOOTER_LINKS_JA = Object.freeze([
  { label: "ヘルプ", href: "/" },
  { label: "利用規約", href: "/" },
  { label: "プライバシー", href: "/" },
  { label: "バグを報告", href: "/" },
  { label: "不快フラグを立てる", href: "/" }
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

/** @param {string | null | undefined} locale */
export const buildWikidotFooterLinks = (locale) => {
  return isJapaneseWikidotLocale(locale) ? WIKIDOT_FOOTER_LINKS_JA : WIKIDOT_FOOTER_LINKS
}

/** @param {string | null | undefined} locale */
export const buildWikidotLoginLabels = (locale) => {
  return isJapaneseWikidotLocale(locale) ? WIKIDOT_LOGIN_LABELS_JA : WIKIDOT_LOGIN_LABELS
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
