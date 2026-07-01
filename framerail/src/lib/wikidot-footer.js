export const WIKIDOT_FOOTER_LINKS = Object.freeze([
  { label: "Help", href: "/" },
  { label: "Terms of Service", href: "/" },
  { label: "Privacy", href: "/" },
  { label: "Report a bug", href: "/" },
  { label: "Flag as objectionable", href: "/" }
])

export const WIKIDOT_POWERED_BY = "Powered by Wikidot.com"

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
export const formatWikidotLicenseName = (licenseName) => {
  const name = `${licenseName ?? ""}`.replace(/\.$/, "").trim()

  if (!name) return "License"

  return /license$/i.test(name) ? name : `${name} License`
}

/**
 * @param {{ licenseName?: string | null; licenseUrl?: string | null }} input
 * @returns {string}
 */
export const buildWikidotLicenseHtml = ({ licenseName, licenseUrl }) => {
  const name = escapeHtml(formatWikidotLicenseName(licenseName))
  const url = escapeHtml(licenseUrl || "/")

  return `Unless otherwise stated, the content of this page is licensed under <a href="${url}">${name}</a>`
}

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
