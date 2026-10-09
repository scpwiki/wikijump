const EN_HEADERS = ["rev.", "", "flags", "actions", "by", "date", "comments"]
const JA_HEADERS = ["rev.", "", "フラグ", "アクション", "by", "日付", "コメント"]

// Frozen anonymous history/PageSourceModule responses from SCP-EN and SCP-JP
// use a div with escaped literal source and <br /> line boundaries. A textarea
// acquires an intrinsic width and content-box padding under Wikidot Base CSS.
// Keep this primitive shared by the AMC response and the Wikidot browser pane.
/** @param {string} source */
export const wikidotRevisionSourceHtml = (source) =>
  `<div class="page-source">${source
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;")
    .replaceAll("\r\n", "\n")
    .replaceAll("\n", "<br />\n")}</div>`

const escapePageSourceHtml = (source) =>
  source
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;")

const SOURCE_COMPONENT_INCLUDE =
  /^([\t ]*\[\[include[\t ]+)(component:[a-z0-9][a-z0-9_-]*)(\]\][\t ]*)$/u
const SOURCE_COMPONENT_INCLUDE_OPEN =
  /^([\t ]*\[\[include[\t ]+)(component:[a-z0-9][a-z0-9_-]*)([\t ]*)$/u

const sourceLineContent = (line) => line.replace(/\r?\n$/u, "")

const parameterBlockEnd = (lines, start) => {
  for (let index = start + 1; index < lines.length; index += 1) {
    const line = sourceLineContent(lines[index])
    if (line.includes("[[") || line.includes("[!--") || line.includes("@@")) return -1
    if (/^[\t ]*\|[A-Za-z_][A-Za-z0-9_]*=[^\n]*\]\][\t ]*$/u.test(line)) return index
    if (/^[\t ]*\|[A-Za-z_][A-Za-z0-9_]*=[^\n]*$/u.test(line)) continue
    if (/^[\t ]*\]\][\t ]*$/u.test(line)) return index
    return -1
  }
  return -1
}

/**
 * Render only the narrowly evidenced same-site component include form as a
 * link. All other source remains escaped text so the read-only source view
 * cannot execute authored markup or guess cross-site routing.
 *
 * @param {string} source
 */
export const wikidotPageSourceHtml = (source) => {
  let inComment = false
  let inCode = false
  const lines = source.match(/[^\n]*\n|[^\n]+$/gu) ?? []

  return lines
    .map((line, index) => {
      const content = sourceLineContent(line)
      const simpleInclude =
        !inComment && !inCode && SOURCE_COMPONENT_INCLUDE.exec(content)
      const openInclude =
        !inComment && !inCode && SOURCE_COMPONENT_INCLUDE_OPEN.exec(content)
      const candidate =
        simpleInclude ??
        (openInclude && parameterBlockEnd(lines, index) >= 0 ? openInclude : null)

      let rendered = escapePageSourceHtml(content)
      if (candidate) {
        const [, before, target, after] = candidate
        const href = `/${encodeURIComponent(target)}`
        rendered = `${escapePageSourceHtml(before)}<a href="${escapePageSourceHtml(href)}">${escapePageSourceHtml(target)}</a>${escapePageSourceHtml(after)}`
      }

      // Source examples and comments stay literal. Do not decorate a matching
      // line until the enclosing region has ended.
      const commentOpen = content.indexOf("[!--")
      const commentClose = content.indexOf("--]")
      if (inComment) {
        if (commentClose >= 0) inComment = false
      } else if (commentOpen >= 0 && (commentClose < 0 || commentClose < commentOpen)) {
        inComment = true
      }

      const codeOpen = content.indexOf("[[code]]")
      const codeClose = content.indexOf("[[/code]]")
      if (inCode) {
        if (codeClose >= 0) inCode = false
      } else if (codeOpen >= 0 && (codeClose < 0 || codeClose < codeOpen)) {
        inCode = true
      }

      return rendered + line.slice(content.length)
    })
    .join("")
}

const FLAG_TITLES = {
  en: {
    create: "New page",
    move: "Page renamed",
    source: "content source text changed",
    tags: "tags changed",
    files: "file/attachment operations"
  },
  ja: {
    create: "新しいページ",
    move: "ページ名が変更されました",
    source: "コンテンツソースが変更されました",
    tags: "タグが変更されました",
    files: "ファイル/添付操作"
  }
}

const ACTION_TITLES = {
  en: {
    view: "View page revision",
    source: "View source of the revision",
    rollback: "Revert to revision"
  },
  ja: {
    view: "リビジョンを閲覧",
    source: "リビジョンのソースを閲覧",
    rollback: "リビジョンの差し戻し"
  }
}

const localeKey = (locale) =>
  String(locale ?? "")
    .toLowerCase()
    .startsWith("ja")
    ? "ja"
    : "en"

export const wikidotHistoryHeaders = (locale) =>
  localeKey(locale) === "ja" ? JA_HEADERS : EN_HEADERS

export const wikidotHistoryActionTitles = (locale) => ACTION_TITLES[localeKey(locale)]

/**
 * Return only revision markers evidenced by the live Wikidot History
 * module. Unknown change fields stay literal/blank instead of being
 * guessed as a generic metadata marker.
 */
export const wikidotRevisionFlags = (revision, locale) => {
  const titles = FLAG_TITLES[localeKey(locale)]
  if (revision.history_kind === "file") return [{ code: "F", title: titles.files }]
  if (revision.revision_type === "create") return [{ code: "N", title: titles.create }]
  if (revision.revision_type === "move") return [{ code: "R", title: titles.move }]

  const changes = new Set(revision.changes ?? [])
  return [
    ...(changes.has("wikitext") ? [{ code: "S", title: titles.source }] : []),
    ...(changes.has("tags") ? [{ code: "A", title: titles.tags }] : []),
    ...(changes.has("files") ? [{ code: "F", title: titles.files }] : [])
  ]
}

export const wikidotRevisionDate = (createdAt) => {
  const date = new Date(createdAt)
  if (!Number.isFinite(date.getTime())) return null
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
  const timestamp = Math.floor(date.getTime() / 1000)
  const day = String(date.getUTCDate()).padStart(2, "0")
  const hour = String(date.getUTCHours()).padStart(2, "0")
  const minute = String(date.getUTCMinutes()).padStart(2, "0")
  return {
    timestamp,
    text: `${day} ${months[date.getUTCMonth()]} ${date.getUTCFullYear()} ${hour}:${minute}`,
    className: `odate time_${timestamp} format_%25e%20%25b%20%25Y%7Cagohover`
  }
}
