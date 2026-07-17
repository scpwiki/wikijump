import { parseDeploymentEnvironment } from "./deployment-environment.js"

const SECURITY_HEADERS = {
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
const CURRENT_SITE_FILE_ORIGIN = "https://wikijump-current-site.invalid"
const DEPLOYMENT_FILE_SUFFIX = {
  local: "localhost",
  dev: "dev",
  prod: "com"
}
const RUNTIME_DEPLOYMENT_ENVIRONMENT = parseDeploymentEnvironment()
const WIKIDOT_INTERWIKI_FRAME_POLICIES = new Map([
  [
    "/-/wikidot-interwiki/interwikiFrame.html",
    "default-src 'none'; script-src 'unsafe-inline'; img-src https://scp-wiki.wdfiles.com; frame-ancestors 'self'"
  ],
  [
    "/-/wikidot-interwiki/styleFrame.html",
    "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; frame-ancestors 'self'"
  ]
])

const shouldSetHsts = () => {
  return RUNTIME_DEPLOYMENT_ENVIRONMENT !== "local"
}

/** @param {unknown} value */
const trustedSiteSlug = (value) => {
  return typeof value === "string" && /^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(value)
    ? value
    : null
}

const materializeExactCspSource = (policy, source, replacement) => {
  let replaced = false
  const directives = policy
    .split(";")
    .map((directive) => directive.trim())
    .filter(Boolean)
    .map((directive) =>
      directive
        .split(/\s+/u)
        .map((token) => {
          if (token !== source) return token
          replaced = true
          return replacement
        })
        .join(" ")
    )
  return replaced ? directives.join("; ") : null
}

/**
 * @param {Response} response
 * @param {string | null | undefined} siteSlug
 */
export const materializeSiteCsp = (response, siteSlug) => {
  const policy = response.headers.get("content-security-policy")
  const slug = trustedSiteSlug(siteSlug)
  if (!policy || !slug) return
  const origin = `https://${slug}.wjfiles.${DEPLOYMENT_FILE_SUFFIX[RUNTIME_DEPLOYMENT_ENVIRONMENT]}`
  const materialized = materializeExactCspSource(policy, CURRENT_SITE_FILE_ORIGIN, origin)
  if (materialized) response.headers.set("content-security-policy", materialized)
}

export const staticSecurityHeaderEntries = () => {
  const headers = Object.entries(SECURITY_HEADERS)

  if (shouldSetHsts()) {
    headers.push(["strict-transport-security", HSTS_HEADER])
  }

  return headers
}

/**
 * @param {Response} response
 * @param {string} pathname
 * @param {string | undefined} siteSlug
 */
export const applyStaticSecurityHeaders = (response, pathname, siteSlug = undefined) => {
  for (const [header, value] of staticSecurityHeaderEntries()) {
    response.headers.set(header, value)
  }

  const framePolicy = WIKIDOT_INTERWIKI_FRAME_POLICIES.get(pathname)
  if (framePolicy) {
    response.headers.set("content-security-policy", framePolicy)
    response.headers.set("x-frame-options", "SAMEORIGIN")
  }
  materializeSiteCsp(response, siteSlug)
}

/**
 * @param {{
 *   setHeader(name: string, value: string): void
 *   removeHeader(name: string): void
 * }} response
 * @param {string} pathname
 */
export const applyStaticSecurityHeadersToNodeResponse = (response, pathname) => {
  for (const [header, value] of staticSecurityHeaderEntries()) {
    response.setHeader(header, value)
  }

  const framePolicy = WIKIDOT_INTERWIKI_FRAME_POLICIES.get(pathname)
  if (framePolicy) {
    response.setHeader("content-security-policy", framePolicy)
    response.setHeader("x-frame-options", "SAMEORIGIN")
  }
}
