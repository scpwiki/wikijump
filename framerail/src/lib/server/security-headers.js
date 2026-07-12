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
const LOCAL_WIKIDOT_INTERWIKI_FRAME_PATHS = new Set([
  "/-/wikidot-interwiki/interwikiFrame.html",
  "/-/wikidot-interwiki/styleFrame.html"
])

const isLocalEnvironment = () => {
  return process.env.FRAMERAIL_ENV === "local" || process.env.NODE_ENV === "development"
}

const shouldSetHsts = () => {
  return !isLocalEnvironment()
}

const allowsLocalWikidotInterwikiFrame = (pathname) => {
  return isLocalEnvironment() && LOCAL_WIKIDOT_INTERWIKI_FRAME_PATHS.has(pathname)
}

export const staticSecurityHeaderEntries = () => {
  const headers = Object.entries(SECURITY_HEADERS)

  if (shouldSetHsts()) {
    headers.push(["strict-transport-security", HSTS_HEADER])
  }

  return headers
}

export const applyStaticSecurityHeaders = (response, pathname) => {
  for (const [header, value] of staticSecurityHeaderEntries()) {
    response.headers.set(header, value)
  }

  if (allowsLocalWikidotInterwikiFrame(pathname)) {
    response.headers.delete("content-security-policy")
    response.headers.delete("x-frame-options")
  }
}

export const applyStaticSecurityHeadersToNodeResponse = (response, pathname) => {
  for (const [header, value] of staticSecurityHeaderEntries()) {
    response.setHeader(header, value)
  }

  if (allowsLocalWikidotInterwikiFrame(pathname)) {
    response.removeHeader("content-security-policy")
    response.removeHeader("x-frame-options")
  }
}
