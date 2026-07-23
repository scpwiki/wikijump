/**
 * @param {unknown} value
 * @returns {string}
 */
const escapeHtml = (value) =>
  String(value).replace(/[&<>"']/g, (character) => {
    switch (character) {
      case "&":
        return "&amp;"
      case "<":
        return "&lt;"
      case ">":
        return "&gt;"
      case '"':
        return "&quot;"
      case "'":
        return "&#39;"
      default:
        return character
    }
  })

/**
 * @param {readonly string[]} tags
 * @param {(tag: string) => string} hrefForTag
 * @returns {string}
 */
export const buildWikidotPageTagsHtml = (
  tags,
  hrefForTag = (tag) => `/system:page-tags/tag/${tag}#pages`
) =>
  tags
    .map((tag) => `<a href="${escapeHtml(hrefForTag(tag))}">${escapeHtml(tag)}</a>`)
    .join("")
