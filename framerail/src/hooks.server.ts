// Hook that runs on every request, including form actions.

import { storeRequestContext } from "$lib/server/load/request-ctx"
import { loadSiteInfo } from "$lib/server/load/site-info"
import type { Handle } from "@sveltejs/kit"

const LOCAL_FILE_IMAGE_SOURCES = ["http://*.wjfiles.localhost:18443"]
const LOCAL_FILE_STYLE_SOURCES = ["http://*.wjfiles.localhost:18443"]
const WIKIDOT_STYLE_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://d3g0gp89917ko0.cloudfront.net",
  "https://fonts.bunny.net",
  "https://maxcdn.bootstrapcdn.com",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]
const WIKIDOT_FONT_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://fonts.bunny.net",
  "https://maxcdn.bootstrapcdn.com",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]

function isLocalEnvironment() {
  return process.env.FRAMERAIL_ENV === "local" || process.env.NODE_ENV === "development"
}

function imageSources() {
  const sources = ["'self'", "data:", "blob:", "https:"]

  if (isLocalEnvironment()) {
    sources.push(...LOCAL_FILE_IMAGE_SOURCES)
  }

  return sources.join(" ")
}

function styleSources() {
  const sources = ["'self'", "'unsafe-inline'", ...WIKIDOT_STYLE_SOURCES]

  if (isLocalEnvironment()) {
    sources.push(...LOCAL_FILE_STYLE_SOURCES)
  }

  return sources.join(" ")
}

function fontSources() {
  return ["'self'", "data:", ...WIKIDOT_FONT_SOURCES].join(" ")
}

const CSP_DIRECTIVES = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "frame-ancestors 'none'",
  "form-action 'self'",
  `img-src ${imageSources()}`,
  `font-src ${fontSources()}`,
  `style-src ${styleSources()}`,
  "script-src 'self' 'unsafe-inline'",
  "connect-src 'self'",
  "worker-src 'self' blob:",
  "manifest-src 'self'"
]

const SECURITY_HEADERS = {
  "content-security-policy": CSP_DIRECTIVES.join("; "),
  "cross-origin-opener-policy": "same-origin",
  "permissions-policy": [
    "accelerometer=()",
    "autoplay=()",
    "camera=()",
    "display-capture=()",
    "encrypted-media=()",
    "fullscreen=(self)",
    "geolocation=()",
    "gyroscope=()",
    "magnetometer=()",
    "microphone=()",
    "midi=()",
    "payment=()",
    "publickey-credentials-get=(self)",
    "screen-wake-lock=()",
    "usb=()",
    "web-share=(self)",
    "xr-spatial-tracking=()"
  ].join(", "),
  "referrer-policy": "strict-origin-when-cross-origin",
  "x-content-type-options": "nosniff",
  "x-frame-options": "DENY"
}

const HSTS_HEADER = "max-age=31536000; includeSubDomains"

function shouldSetHsts() {
  return !isLocalEnvironment()
}

export const handle: Handle = async ({ event, resolve }) => {
  const { request, cookies, locals, params } = event

  // Gather common request metadata into a shared context.
  const { siteId } = loadSiteInfo(request.headers)
  const page_slug = params.slug
  const sessionToken = cookies.get("wikijump_token")

  storeRequestContext(locals, sessionToken, siteId, page_slug)

  // Continue processing the request
  const response = await resolve(event)

  for (const [header, value] of Object.entries(SECURITY_HEADERS)) {
    response.headers.set(header, value)
  }

  if (shouldSetHsts()) {
    response.headers.set("strict-transport-security", HSTS_HEADER)
  }

  return response
}
