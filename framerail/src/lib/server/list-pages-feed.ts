const FEED_PATH_PREFIX = "/feed/pages/"
const MAX_FEED_PATH_BYTES = 64 * 1024
const MAX_FEED_PATH_PAIRS = 100

const FEED_SELECTOR_KEYS = [
  "pagetype",
  "category",
  "tags",
  "parent",
  "created_by",
  "rating",
  "range"
] as const

export interface WikidotListPagesFeedSelectors {
  pagetype?: string
  category?: string
  tags?: string
  parent?: string
  created_by?: string
  rating?: string
  range?: string
}

export interface ParsedWikidotListPagesFeedPath {
  title: string
  description: string
  home: string | null
  selectors: WikidotListPagesFeedSelectors
}

export interface WikidotListPagesFeedItem {
  slug: string
  title: string
  created_at: string
  body_html: string
  created_by_html: string
}

export interface WikidotListPagesFeedOutput {
  items: WikidotListPagesFeedItem[]
}

export class InvalidWikidotListPagesFeedPath extends Error {}

function decodeFeedPathSegment(segment: string): string {
  try {
    return decodeURIComponent(segment.replaceAll("+", " "))
  } catch {
    throw new InvalidWikidotListPagesFeedPath(
      "Malformed percent escape in Wikidot ListPages feed path"
    )
  }
}

export function parseWikidotListPagesFeedPath(
  requestUrl: string
): ParsedWikidotListPagesFeedPath | null {
  const pathname = new URL(requestUrl).pathname
  if (!pathname.startsWith(FEED_PATH_PREFIX)) return null

  const encodedPath = pathname.slice(FEED_PATH_PREFIX.length)
  if (new TextEncoder().encode(encodedPath).length > MAX_FEED_PATH_BYTES) {
    throw new InvalidWikidotListPagesFeedPath("Wikidot ListPages feed path is too long")
  }

  const segments = encodedPath.split("/")
  if (Math.ceil(segments.length / 2) > MAX_FEED_PATH_PAIRS) {
    throw new InvalidWikidotListPagesFeedPath(
      "Wikidot ListPages feed path has too many arguments"
    )
  }

  const values = new Map<string, string>()
  for (let index = 0; index < segments.length; index += 2) {
    const key = decodeFeedPathSegment(segments[index] ?? "")
    const value = decodeFeedPathSegment(segments[index + 1] ?? "")
    values.set(key, value)
  }

  const selectors: WikidotListPagesFeedSelectors = {}
  for (const key of FEED_SELECTOR_KEYS) {
    const value = values.get(key)
    if (value !== undefined && value !== "") selectors[key] = value
  }

  const home = values.get("h")
  return {
    title: values.get("t") ?? "",
    description: values.get("d") ?? "",
    home: home ? home : null,
    selectors
  }
}

export function wikidotListPagesFeedSelectorError(
  selectors: WikidotListPagesFeedSelectors
): string | null {
  if (
    selectors.pagetype !== undefined &&
    !["*", "hidden", "normal"].includes(selectors.pagetype)
  ) {
    return "Invalid pagetype attribute."
  }
  if (
    selectors.rating !== undefined &&
    !/^(?:>=|<=|<>|>|<|=)?-?\d+$/.test(selectors.rating.trim())
  ) {
    return "Invalid rating argument."
  }
  if (selectors.range !== undefined && selectors.range !== ".") {
    return "Invalid range argument."
  }
  return null
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;")
}

function cdata(value: string): string {
  return value.replaceAll("]]>", "]]]]><![CDATA[>")
}

export function formatWikidotFeedDate(value: Date): string {
  const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec"
  ]
  const day = value.getUTCDate().toString().padStart(2, "0")
  const hours = value.getUTCHours().toString().padStart(2, "0")
  const minutes = value.getUTCMinutes().toString().padStart(2, "0")
  const seconds = value.getUTCSeconds().toString().padStart(2, "0")
  return `${weekdays[value.getUTCDay()]}, ${day} ${months[value.getUTCMonth()]} ${value.getUTCFullYear()} ${hours}:${minutes}:${seconds} +0000`
}

function feedItemXml(origin: string, item: WikidotListPagesFeedItem): string {
  const pageUrl = `${origin}/${item.slug}`
  const author = `<p>by ${item.created_by_html}</p>`
  const content = `${item.body_html}\n${author}`
  return [
    "\t\t<item>",
    `\t\t\t<guid>${escapeXml(pageUrl)}</guid>`,
    `\t\t\t<title>${escapeXml(item.title)}</title>`,
    `\t\t\t<link>${escapeXml(pageUrl)}</link>`,
    `\t\t\t<description>${escapeXml(content)}</description>`,
    `\t\t\t<pubDate>${formatWikidotFeedDate(new Date(item.created_at))}</pubDate>`,
    `\t\t\t<content:encoded><![CDATA[${cdata(content)}]]></content:encoded>`,
    "\t\t</item>"
  ].join("\n")
}

export function buildWikidotListPagesFeedXml(
  requestUrl: string,
  path: ParsedWikidotListPagesFeedPath,
  output: WikidotListPagesFeedOutput,
  now = new Date()
): string {
  const origin = new URL(requestUrl).origin
  const items = output.items.map((item) => feedItemXml(origin, item)).join("\n")
  return [
    '<?xml version="1.0" encoding="UTF-8" ?>',
    '<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikidot="http://www.wikidot.com/rss-namespace">',
    "",
    "\t<channel>",
    `\t\t<title>${escapeXml(path.title)}</title>`,
    `\t\t<link>${escapeXml(path.home ?? origin)}</link>`,
    `\t\t<description>${escapeXml(path.description)}</description>`,
    "\t\t<copyright></copyright>",
    `\t\t<lastBuildDate>${formatWikidotFeedDate(now)}</lastBuildDate>`,
    items,
    "\t</channel>",
    "</rss>"
  ]
    .filter((line) => line !== "")
    .join("\n")
}

export function wikidotListPagesFeedErrorBody(message: string): string {
  return (
    "A nasty error has occurred. If the problem repeats, please fill " +
    `(if possible) a bug report.<br/><br/>${escapeXml(message)}`
  )
}

export const WIKIDOT_LIST_PAGES_FEED_HEADERS = {
  "cache-control": "no-cache, must-revalidate",
  "content-type": "text/xml;charset=utf-8",
  expires: "Mon, 26 Jul 1997 05:00:00 GMT"
} as const
