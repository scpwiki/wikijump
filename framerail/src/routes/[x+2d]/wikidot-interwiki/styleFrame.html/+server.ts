import type { RequestHandler } from "./$types"

const FRAME_HEADERS = {
  "content-type": "text/html; charset=utf-8",
  "cache-control": "no-store"
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
}

export const GET: RequestHandler = ({ url }) => {
  const css = (url.searchParams.get("css") ?? "").replace(
    /<\/(style|script)/giu,
    "<\\/$1"
  )
  const theme = url.searchParams.get("theme") ?? ""
  const priority = url.searchParams.get("priority") ?? ""
  const body = `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Local Wikidot style frame</title>
    <meta name="wikidot-style-priority" content="${escapeHtml(priority)}">
    <meta name="wikidot-style-theme" content="${escapeHtml(theme)}">
    <style>${css}</style>
  </head>
  <body></body>
</html>
`

  return new Response(body, { headers: FRAME_HEADERS })
}

export const HEAD: RequestHandler = () => new Response(null, { headers: FRAME_HEADERS })
