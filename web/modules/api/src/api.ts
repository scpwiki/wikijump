import { Api } from "../vendor/api"

export class WikijumpAPI extends Api<void> {
  // TODO: allow giving a specific site here
  constructor(baseUrl = "/api--v0") {
    super({
      baseUrl,
      baseApiParams: {
        // kind of a hack, but using a getter ensures that the API is always
        // called with up to date CSRF data
        get headers(): Record<string, string> {
          const csrfMeta = getCSRFMeta()
          const csrfCookie = getCSRFCookie()
          if (csrfCookie) {
            return {
              "X-CSRF-TOKEN": csrfMeta,
              "X-XSRF-TOKEN": csrfCookie
            }
          } else {
            return {
              "X-CSRF-TOKEN": csrfMeta
            }
          }
        },
        format: "json"
      }
    })
  }
}

/**
 * Retrieves the CSRF token from the `<meta name="csrf-token" ...>` tag in
 * the `<head>`. This should always be present, so this function throws if
 * that element can't be found.
 */
function getCSRFMeta() {
  const meta = document.head.querySelector("meta[name=csrf-token]")
  if (!meta) throw new Error("No CSRF meta tag found")
  return meta.getAttribute("content")!
}

/** Retrieves the CSRF token from the `XSRF-TOKEN` cookie, if it exists. */
function getCSRFCookie() {
  const value = document.cookie
    .split(/;\s*/)
    .find(c => c.startsWith("XSRF-TOKEN="))
    ?.split("=")[1]
  return value
}
