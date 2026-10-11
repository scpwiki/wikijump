import { formatWikidotFeedDate } from "./list-pages-feed.ts"

/**
 * The `/feed/site-changes.xml` RSS serializer.
 *
 * Current Wikidot serves a real RSS 2.0 document here that reports the same
 * recent page activity as the SiteChanges module: channel title and site link,
 * newest-first bounded change rows, per-item revision identity/change
 * type/author, and a page content preview. All internal links use the local
 * Wikijump origin rather than the source Wikidot host.
 */

export interface WikidotSiteChangesFeedItem {
  revision_id: number
  revision_number: number
  is_new_page: boolean
  is_file: boolean
  created_at: string
  slug: string
  title: string
  change_type: string
  user_id: number
  user_name: string
  body_html: string
}

export interface WikidotSiteChangesFeedOutput {
  site_name: string
  items: WikidotSiteChangesFeedItem[]
}

export const WIKIDOT_SITE_CHANGES_FEED_HEADERS = {
  "cache-control": "no-cache, must-revalidate",
  "content-type": "text/xml;charset=utf-8",
  expires: "Mon, 26 Jul 1997 05:00:00 GMT"
} as const

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

export function formatWikidotSiteChangesFeedDate(value: Date): string {
  return formatWikidotFeedDate(value)
}

function itemXml(origin: string, item: WikidotSiteChangesFeedItem): string {
  const pageUrl = `${origin}/${item.slug}`
  // The GUID reports the page identity together with its revision, so a
  // subscriber can tell a new revision from an unchanged page.
  const guidOrigin = new URL(origin)
  guidOrigin.protocol = "http:"
  const guid = `${guidOrigin.origin}/${item.slug}#revision-${item.revision_id}`
  const revisionLabel = item.is_new_page ? "new" : `rev. ${item.revision_number}`
  const description = [
    `Page: <a href="${pageUrl}">${item.title}</a>`,
    `Current revision number: ${revisionLabel}`,
    `Date changed: ${formatWikidotSiteChangesFeedDate(new Date(item.created_at))}`,
    `Change type: ${item.change_type}`,
    `By: ${item.user_name}`,
    `Page content preview: ${item.body_html}`
  ].join("<br/>")
  const title = `"${item.title}" - ${item.change_type} change`

  return [
    "\t\t<item>",
    `\t\t\t<guid>${escapeXml(guid)}</guid>`,
    `\t\t\t<title>${escapeXml(title)}</title>`,
    `\t\t\t<link>${escapeXml(pageUrl)}</link>`,
    `\t\t\t<description>${escapeXml(description)}</description>`,
    `\t\t\t<pubDate>${formatWikidotSiteChangesFeedDate(new Date(item.created_at))}</pubDate>`,
    "\t\t\t<content:encoded>",
    "\t\t\t\t<![CDATA[",
    `\t\t\t\t${cdata(item.body_html)}`,
    "\t\t\t\t]]>",
    "\t\t\t</content:encoded>",
    "\t\t</item>"
  ].join("\n")
}

export function buildWikidotSiteChangesFeedXml(
  requestUrl: string,
  output: WikidotSiteChangesFeedOutput,
  now = new Date()
): string {
  const origin = new URL(requestUrl).origin
  const items = output.items.map((item) => itemXml(origin, item)).join("\n")
  return [
    '<?xml version="1.0" encoding="UTF-8" ?>',
    '<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikidot="http://www.wikidot.com/rss-namespace">',
    "",
    "\t<channel>",
    `\t\t<title>Recent page changes from site "${escapeXml(output.site_name)}" (a Wikidot site)</title>`,
    `\t\t<link>${escapeXml(origin)}</link>`,
    `\t\t<lastBuildDate>${formatWikidotSiteChangesFeedDate(now)}</lastBuildDate>`,
    ...(items ? [items] : []),
    "\t</channel>",
    "</rss>"
  ].join("\n")
}
