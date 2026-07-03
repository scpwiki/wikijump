const CROM_INTERWIKI_QUERY = `
query InterwikiQuery($url: URL!) {
  page(url: $url) {
    translations {
      url
    }
    translationOf {
      url
      translations {
        url
      }
    }
  }
}
`

const DEFAULT_ICON_URL = "//scp-wiki.wdfiles.com/local--files/nav:side/default.png"

/**
 * @typedef {{ label: string; heading: string; url: string }} WikidotInterwikiBranch
 *
 *
 * @typedef {Readonly<Record<string, WikidotInterwikiBranch>>} WikidotInterwikiCommunity
 *
 *
 * @typedef {Readonly<Record<string, WikidotInterwikiCommunity>>} WikidotInterwikiCommunities
 *
 *
 * @typedef {{
 *   code: string
 *   label: string
 *   href: string
 *   original: boolean
 * }} WikidotInterwikiLink
 */

/** @type {WikidotInterwikiCommunities} */
export const WIKIDOT_INTERWIKI_BRANCHES = Object.freeze({
  scp: Object.freeze({
    cn: Object.freeze({
      label: "中文",
      heading: "其他语言",
      url: "https://scp-wiki-cn.wikidot.com/"
    }),
    cs: Object.freeze({
      label: "Český",
      heading: "V jiných jazycích",
      url: "https://scp-cs.wikidot.com/"
    }),
    en: Object.freeze({
      label: "English",
      heading: "In other languages",
      url: "https://scp-wiki.wikidot.com/"
    }),
    fr: Object.freeze({
      label: "Français",
      heading: "Dans d’autres langues",
      url: "https://fondationscp.wikidot.com/"
    }),
    de: Object.freeze({
      label: "Deutsch",
      heading: "In anderen Sprachen",
      url: "https://scp-wiki-de.wikidot.com/"
    }),
    int: Object.freeze({
      label: "International",
      heading: "Languages",
      url: "https://scp-int.wikidot.com/"
    }),
    it: Object.freeze({
      label: "Italiano",
      heading: "In altre lingue",
      url: "https://fondazionescp.wikidot.com/"
    }),
    jp: Object.freeze({
      label: "日本語",
      heading: "他言語版",
      url: "https://scp-jp.wikidot.com/"
    }),
    ko: Object.freeze({
      label: "한국어",
      heading: "다른 언어",
      url: "https://scpko.wikidot.com/"
    }),
    pl: Object.freeze({
      label: "Polski",
      heading: "W innych językach",
      url: "http://scp-pl.wikidot.com/"
    }),
    ptbr: Object.freeze({
      label: "Português",
      heading: "Em outros idiomas",
      url: "https://scp-pt-br.wikidot.com/"
    }),
    ru: Object.freeze({
      label: "Русский",
      heading: "На других языках",
      url: "https://scpfoundation.net/"
    }),
    es: Object.freeze({
      label: "Español",
      heading: "En otros idiomas",
      url: "http://lafundacionscp.wikidot.com/"
    }),
    th: Object.freeze({
      label: "ภาษาไทย",
      heading: "ภาษาอื่น",
      url: "https://scp-th.wikidot.com/"
    }),
    ua: Object.freeze({
      label: "Українська",
      heading: "Іншими мовами",
      url: "https://scp-ukrainian.wikidot.com/"
    }),
    "zh-tr": Object.freeze({
      label: "繁體中文",
      heading: "其他語言",
      url: "https://scp-zh-tr.wikidot.com/"
    }),
    vn: Object.freeze({
      label: "Tiếng Việt",
      heading: "Ngôn ngữ",
      url: "https://scp-vn.wikidot.com/"
    })
  }),
  wl: Object.freeze({
    cn: Object.freeze({
      label: "中文",
      heading: "其他语言",
      url: "https://scp-wiki-cn.wikidot.com/"
    }),
    cs: Object.freeze({
      label: "Čeština",
      heading: "V jiných jazycích",
      url: "https://wanderers-library-cs.wikidot.com/"
    }),
    en: Object.freeze({
      label: "English",
      heading: "Languages",
      url: "https://wanderers-library.wikidot.com/"
    }),
    fr: Object.freeze({
      label: "Français",
      heading: "Dans d’autres langues",
      url: "https://fondationscp.wikidot.com/"
    }),
    jp: Object.freeze({
      label: "日本語",
      heading: "他言語版",
      url: "https://wanderers-library-jp.wikidot.com/"
    }),
    ko: Object.freeze({
      label: "한국어",
      heading: "다른 언어",
      url: "https://wanderers-library-ko.wikidot.com/"
    }),
    pl: Object.freeze({
      label: "Polski",
      heading: "W innych językach",
      url: "https://wanderers-library-pl.wikidot.com/"
    }),
    ru: Object.freeze({
      label: "Русский",
      heading: "На других языках",
      url: "https://scpfoundation.net/"
    })
  })
})

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
 * @param {string} url
 * @returns {string}
 */
const cromComparableUrl = (url) => url.replace(/^https:/i, "http:")

/**
 * @param {string} url
 * @returns {boolean}
 */
const isAllowedCromUrl = (url) => {
  try {
    const hostname = new URL(url).hostname
    return hostname.endsWith(".wikidot.com") || hostname === "scpfoundation.net"
  } catch {
    return false
  }
}

/**
 * @param {string} sourcePath
 * @returns {string}
 */
const normalizeSourcePath = (sourcePath) => {
  return sourcePath.replace(/^_default:/, "").replace(/^\/+/, "")
}

/**
 * @param {string} community
 * @returns {WikidotInterwikiCommunity}
 */
const branchesForCommunity = (community) => {
  return WIKIDOT_INTERWIKI_BRANCHES[community] ?? Object.freeze({})
}

/**
 * @param {WikidotInterwikiCommunity} branches
 * @param {string} url
 * @returns {{
 *   code: string
 *   label: string
 *   heading: string
 *   url: string
 * } | null}
 */
const findBranchForUrl = (branches, url) => {
  const comparable = cromComparableUrl(url)

  for (const [code, branch] of Object.entries(branches)) {
    if (comparable.startsWith(cromComparableUrl(branch.url))) {
      return { code, ...branch }
    }
  }

  return null
}

/**
 * @param {{ url?: string | null } | null | undefined} item
 * @returns {string | null}
 */
const itemUrl = (item) => {
  return typeof item?.url === "string" ? item.url : null
}

/**
 * @param {{
 *       translations?: { url?: string | null }[] | null
 *       translationOf?: {
 *         url?: string | null
 *         translations?: { url?: string | null }[] | null
 *       } | null
 *     }
 *   | null
 *   | undefined} page
 * @returns {string[]}
 */
const collectTranslationUrls = (page) => {
  const urls = []

  for (const item of page?.translations ?? []) {
    const url = itemUrl(item)
    if (url) urls.push(url)
  }

  const originalUrl = itemUrl(page?.translationOf)
  if (originalUrl) urls.push(originalUrl)

  for (const item of page?.translationOf?.translations ?? []) {
    const url = itemUrl(item)
    if (url) urls.push(url)
  }

  return urls
}

/**
 * @param {{ community: string; lang: string; sourcePath: string }} input
 * @returns {string | null}
 */
export const buildWikidotInterwikiSourceUrl = ({ community, lang, sourcePath }) => {
  const branch = branchesForCommunity(community)[lang]
  if (!branch) return null

  const path = normalizeSourcePath(sourcePath)
  const url = `${branch.url}${path}`
  return isAllowedCromUrl(url) ? url : null
}

/**
 * @param {{
 *   community: string
 *   lang: string
 *   sourcePath: string
 *   page?: Parameters<typeof collectTranslationUrls>[0]
 * }} input
 * @returns {WikidotInterwikiLink[]}
 */
export const extractWikidotInterwikiLinks = ({ community, lang, sourcePath, page }) => {
  const branches = branchesForCommunity(community)
  const currentUrl = buildWikidotInterwikiSourceUrl({ community, lang, sourcePath })
  const seenCodes = new Set()
  /** @type {WikidotInterwikiLink[]} */
  const links = []

  if (!currentUrl) return links

  const currentComparable = cromComparableUrl(currentUrl)

  for (const href of collectTranslationUrls(page)) {
    if (cromComparableUrl(href).startsWith(currentComparable)) continue

    const branch = findBranchForUrl(branches, href)
    if (!branch || seenCodes.has(branch.code)) continue

    seenCodes.add(branch.code)
    links.push({
      code: branch.code,
      label: branch.label,
      href,
      original: page?.translationOf?.url === href
    })
  }

  return links.sort((left, right) =>
    left.code < right.code ? -1 : left.code > right.code ? 1 : 0
  )
}

/**
 * @param {{
 *   community: string
 *   lang: string
 *   pagename: string
 *   page?: Parameters<typeof collectTranslationUrls>[0] | null
 * }} input
 * @returns {string}
 */
export const buildWikidotInterwikiFrameHtml = ({ community, lang, pagename, page }) => {
  const branches = branchesForCommunity(community)
  const branch = branches[lang]
  const links = extractWikidotInterwikiLinks({
    community,
    lang,
    sourcePath: pagename,
    page
  })
  const heading = branch?.heading ?? ""
  const menuItems = links
    .map((link) => {
      const originalClass = link.original ? " original" : ""
      return `<div class="menu-item${originalClass}" name="${escapeHtml(link.code)}"><img src="${DEFAULT_ICON_URL}" alt="default.png" class="image"><a href="${escapeHtml(link.href)}" target="_parent">${escapeHtml(link.label)}</a></div>`
    })
    .join(" ")
  const display = links.length > 0 ? "" : ' style="display: none"'

  return `<!DOCTYPE html>
<html id="interwiki" style="min-width: max-content">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <title>Local Wikidot interwiki frame</title>
    <script>
      const resizeLocalInterwikiFrame = () => {
        try {
          const frame = window.frameElement;
          if (frame) frame.style.height = document.documentElement.scrollHeight + "px";
        } catch {
          /* Cross-origin embedding cannot use the local same-origin resize path. */
        }
      };
      addEventListener("DOMContentLoaded", resizeLocalInterwikiFrame);
      addEventListener("load", resizeLocalInterwikiFrame);
    </script>
  </head>
  <body data-lang="${escapeHtml(lang)}" data-community="${escapeHtml(community)}" data-pagename="${escapeHtml(pagename)}">
    <div class="side-block"${display}>
      <div class="heading">
        <p>${escapeHtml(heading)}</p>
      </div>
      ${menuItems}
    </div>
    <div id="resizer-container"></div>
  </body>
</html>
`
}

/**
 * @param {typeof fetch} fetchImpl
 * @param {string} sourceUrl
 * @returns {Promise<Parameters<typeof collectTranslationUrls>[0] | null>}
 */
export const fetchCromInterwikiPage = async (fetchImpl, sourceUrl) => {
  if (!isAllowedCromUrl(sourceUrl)) return null

  const queryUrl = new URL("https://api.crom.avn.sh/graphql")
  queryUrl.searchParams.set("query", CROM_INTERWIKI_QUERY)
  queryUrl.searchParams.set(
    "variables",
    JSON.stringify({ url: cromComparableUrl(sourceUrl) })
  )

  try {
    const response = await fetchImpl(queryUrl, {
      headers: {
        accept: "application/json"
      }
    })

    if (!response.ok) return null

    const payload = await response.json()
    return payload?.data?.page ?? null
  } catch {
    return null
  }
}
