import { Api } from "../vendor/api"

const API_PATH = "/api--v0"

/**
 * Wikijump API class. You usually don't need to make your own instance of
 * this class, as the instance {@link WikijumpAPI} has already been
 * constructed for you.
 */
export class WikijumpAPIInstance extends Api<void> {
  // TODO: allow giving a specific site here
  /** @param headers - Extra headers to send with every request. */
  constructor(headers: Record<string, string> = {}) {
    super({
      baseUrl: API_PATH,
      baseApiParams: {
        headers: {
          "Accept": "application/json",
          "Content-Type": "application/json",
          "X-CSRF-TOKEN": getCSRFMeta(),
          ...headers
        },
        secure: true,
        format: "json"
      },
      securityWorker() {
        const csrf = getCSRFCookie()
        if (csrf) return { headers: { "X-XSRF-TOKEN": csrf } }
      }
    })
  }
}

/** Default Wikijump API instance. */
export const WikijumpAPI = new WikijumpAPIInstance()

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
