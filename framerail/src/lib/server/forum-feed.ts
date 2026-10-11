export type WikidotForumFeedKind = "threads" | "posts"
export type WikidotForumFeedScope = "site" | "category" | "thread"

/**
 * A Wikidot forum feed is either the site-wide pair or one named forum object.
 *
 * `ct-<id>` lists the new threads of a category, `cp-<id>` its new posts, and
 * `t-<id>` the posts of a single thread. The ID is the identifier Wikidot
 * exposes in the HTML forum routes; imported objects keep their legacy Wikidot
 * ID, so locally created objects resolve through the same space.
 */
export interface WikidotForumFeedTarget {
  kind: WikidotForumFeedKind
  categoryId?: number
  threadId?: number
}

const MAX_FEED_ID = 9_007_199_254_740_992

function parseFeedId(raw: string): number | null {
  if (!/^\d{1,16}$/.test(raw)) return null
  const id = Number(raw)
  if (!Number.isSafeInteger(id) || id <= 0 || id > MAX_FEED_ID) return null
  return id
}

export function parseWikidotForumFeed(feed: string): WikidotForumFeedTarget | null {
  const name = feed.toLowerCase()
  const suffix = ".xml"
  if (!name.endsWith(suffix)) return null
  const target = name.slice(0, -suffix.length)

  if (target === "threads") return { kind: "threads" }
  if (target === "posts") return { kind: "posts" }

  for (const prefix of ["ct-", "cp-"] as const) {
    if (!target.startsWith(prefix)) continue
    const id = parseFeedId(target.slice(prefix.length))
    if (id === null) return null
    return { kind: prefix === "ct-" ? "threads" : "posts", categoryId: id }
  }

  if (target.startsWith("t-")) {
    const id = parseFeedId(target.slice("t-".length))
    if (id === null) return null
    return { kind: "posts", threadId: id }
  }

  return null
}

export interface WikidotForumFeedItem {
  forum_post_id: number
  forum_thread_id: number
  created_at: string
  title: string
  thread_title: string
  thread_slug: string
  page_slug: string | null
  forum_group_id: number
  forum_group_name: string
  forum_category_id: number
  forum_category_name: string
  forum_category_slug: string
  author_user_id: number
  author_name: string
  content_html: string
}

export interface WikidotForumFeedOutput {
  scope: WikidotForumFeedScope
  site_name: string
  site_description: string
  forum_group_name?: string
  forum_category_id?: number
  forum_category_name?: string
  forum_thread_id?: number
  forum_thread_title?: string
  items: WikidotForumFeedItem[]
}

const XML_HEADERS = {
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

export function formatWikidotForumFeedDate(value: Date): string {
  const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
  const day = value.getUTCDate().toString().padStart(2, "0")
  const hours = value.getUTCHours().toString().padStart(2, "0")
  const minutes = value.getUTCMinutes().toString().padStart(2, "0")
  const seconds = value.getUTCSeconds().toString().padStart(2, "0")
  return `${weekdays[value.getUTCDay()]}, ${day} ${months[value.getUTCMonth()]} ${value.getUTCFullYear()} ${hours}:${minutes}:${seconds} +0000`
}

function itemXml(origin: string, kind: WikidotForumFeedKind, item: WikidotForumFeedItem): string {
  const threadPath = item.page_slug
    ? `/${item.page_slug}/comments/show`
    : `/forum/t-${item.forum_thread_id}/${item.thread_slug}`
  const postPath = `${threadPath}#post-${item.forum_post_id}`
  const isThread = kind === "threads"
  const canonicalPath = `/forum/t-${item.forum_thread_id}`
  const guidOrigin = new URL(origin)
  guidOrigin.protocol = "http:"
  const guid = isThread ? canonicalPath : `${canonicalPath}#post-${item.forum_post_id}`
  const link = isThread
    ? `${canonicalPath}/${item.thread_slug}`
    : postPath
  const title = isThread ? item.thread_title : ""
  const content = isThread
    ? item.content_html
    : `${item.content_html}\n<br/>Forum category: <a href="${origin}/forum/c-${item.forum_category_id}">${escapeXml(item.forum_group_name)} / ${escapeXml(item.forum_category_name)}</a><br/>Forum thread: <a href="${origin}${canonicalPath}/${item.thread_slug}">${escapeXml(item.thread_title)}</a>`

  return [
    "\t\t<item>",
    `\t\t\t<guid>${escapeXml(`${guidOrigin.origin}${guid}`)}</guid>`,
    title ? `\t\t\t<title>${escapeXml(title)}</title>` : "\t\t\t<title/>",
    `\t\t\t<link>${escapeXml(`${origin}${link}`)}</link>`,
    "\t\t\t<description/>",
    `\t\t\t<pubDate>${formatWikidotForumFeedDate(new Date(item.created_at))}</pubDate>`,
    `\t\t\t<wikidot:authorName>${escapeXml(item.author_name)}</wikidot:authorName>`,
    `\t\t\t<wikidot:authorUserId>${item.author_user_id}</wikidot:authorUserId>`,
    "\t\t\t<content:encoded>",
    "\t\t\t\t<![CDATA[",
    `\t\t\t\t${cdata(content)}`,
    "\t\t\t\t]]>",
    "\t\t\t</content:encoded>",
    "\t\t</item>"
  ].join("\n")
}

/**
 * The channel Wikidot publishes for each feed scope.
 *
 * The site-wide pair is a frozen live observation. The scoped routes reuse that
 * observed grammar with the named scope appended, and the per-thread title uses
 * the observed `Comments for page "..."` spelling, where a page discussion
 * thread carries the page title.
 */
function forumFeedChannel(
  origin: string,
  output: WikidotForumFeedOutput,
  kind: WikidotForumFeedKind
): { title: string; link: string; description: string } {
  const kindWord = kind === "threads" ? "Threads" : "Posts"
  const feedWord = kind === "threads" ? "threads" : "posts"
  if (output.scope === "thread") {
    const thread = output.forum_thread_title ?? ""
    return {
      title: `Comments for page "${thread}"`,
      link: `${origin}/forum/t-${output.forum_thread_id ?? 0}`,
      description: `${kindWord} in forum thread "${thread}" of the site "${output.site_name}" - ${output.site_description}`
    }
  }
  if (output.scope === "category") {
    const category = `${output.forum_group_name ?? ""} / ${output.forum_category_name ?? ""}`
    return {
      title: `${output.site_name} - new forum ${feedWord} in category "${category}"`,
      link: `${origin}/forum/c-${output.forum_category_id ?? 0}`,
      description: `${kindWord} in forum category "${category}" of the site "${output.site_name}" - ${output.site_description}`
    }
  }
  return {
    title: `${output.site_name} - new forum ${feedWord}`,
    link: `${origin}/forum/start`,
    description: `${kindWord} in forums of the site "${output.site_name}" - ${output.site_description}`
  }
}

export function buildWikidotForumFeedXml(
  requestUrl: string,
  kind: WikidotForumFeedKind,
  output: WikidotForumFeedOutput,
  now = new Date()
): string {
  const origin = new URL(requestUrl).origin
  const channel = forumFeedChannel(origin, output, kind)
  const items = output.items.map((item) => itemXml(origin, kind, item)).join("\n")
  return [
    '<?xml version="1.0" encoding="UTF-8" ?>',
    '<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikidot="http://www.wikidot.com/rss-namespace">',
    "",
    "\t<channel>",
    `\t\t<title>${escapeXml(channel.title)}</title>`,
    `\t\t<link>${escapeXml(channel.link)}</link>`,
    `\t\t<description>${escapeXml(channel.description)}</description>`,
    "\t\t<copyright/>",
    `\t\t<lastBuildDate>${formatWikidotForumFeedDate(now)}</lastBuildDate>`,
    ...(items ? [items] : []),
    "\t</channel>",
    "</rss>"
  ].join("\n")
}

export function wikidotForumFeedHeaders(): Headers {
  return new Headers(XML_HEADERS)
}
