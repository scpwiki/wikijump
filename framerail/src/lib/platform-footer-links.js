/**
 * Native Wikijump footer destinations.
 *
 * `Docs`, `Terms`, `Privacy` and `Security` are platform-global actions,
 * not wiki content. They never resolve against the wiki a reader is
 * viewing: the current site's origin is never consulted, so no wiki can
 * capture a footer click, and each item leads somewhere of its own instead
 * of back to the current homepage. That is the same distinction Wikidot
 * draws between the site it serves and its own platform footer (see
 * `$lib/wikidot/wikidot-footer`), where Help, Terms of Service and Privacy
 * point at `www.wikidot.com` and never at the current site root.
 *
 * Destinations are instance-level configuration. A deployment points each
 * item at its own docs, legal and security resources with the
 * `PUBLIC_WIKIJUMP_FOOTER_*_URL` environment variables, so no instance is
 * forced to send its readers to another operator's hostname.
 *
 * Every destination is an absolute `https:` URL. A configured value that
 * is missing, non-https, relative, or carries credentials is a
 * configuration defect and fails closed to the documented default, because
 * a relative or scheme-relative destination would silently resolve against
 * the viewed wiki's origin.
 */

const PLATFORM_FOOTER_ORIGIN = "https://wikijump.com"

/**
 * Ordered native footer items. The keys name the internationalization keys
 * that supply each label, and the order is the Wikijump footer order.
 */
export const NATIVE_FOOTER_DESTINATION_KEYS = Object.freeze([
  "terms-conditions",
  "privacy",
  "docs",
  "security"
])

/**
 * Environment variables a deployment sets to override one native footer
 * destination.
 */
export const NATIVE_FOOTER_DESTINATION_ENVIRONMENT_VARIABLES = Object.freeze({
  "terms-conditions": "PUBLIC_WIKIJUMP_FOOTER_TERMS_CONDITIONS_URL",
  privacy: "PUBLIC_WIKIJUMP_FOOTER_PRIVACY_URL",
  docs: "PUBLIC_WIKIJUMP_FOOTER_DOCS_URL",
  security: "PUBLIC_WIKIJUMP_FOOTER_SECURITY_URL"
})

/** Documented destinations for the public Wikijump instance. */
export const DEFAULT_NATIVE_FOOTER_LINKS = Object.freeze([
  Object.freeze({
    key: "terms-conditions",
    href: `${PLATFORM_FOOTER_ORIGIN}/legal/terms-of-service`
  }),
  Object.freeze({
    key: "privacy",
    href: `${PLATFORM_FOOTER_ORIGIN}/legal/privacy-policy`
  }),
  Object.freeze({ key: "docs", href: `${PLATFORM_FOOTER_ORIGIN}/docs` }),
  Object.freeze({ key: "security", href: `${PLATFORM_FOOTER_ORIGIN}/legal/security` })
])

const MAX_DESTINATION_LENGTH = 2048
const UNSAFE_CHARACTER = /[\u0000-\u0020\u007f\\]/u

/**
 * Accept only a complete, safe web destination. A configured value that is
 * missing, non-https, relative, or carries credentials is rejected so the
 * caller can fall back to the documented default instead of emitting a
 * link the viewed wiki could influence.
 *
 * @param {string | null | undefined} value
 * @returns {string | null} The normalized destination, or `null` when the
 *   value is unusable.
 */
export const normalizePlatformFooterDestination = (value) => {
  if (typeof value !== "string") return null

  const trimmed = value.trim()
  if (trimmed.length === 0 || trimmed.length > MAX_DESTINATION_LENGTH) return null
  if (UNSAFE_CHARACTER.test(trimmed)) return null

  try {
    const url = new URL(trimmed)
    if (url.protocol !== "https:") return null
    if (!url.hostname || url.username || url.password) return null
    return url.href
  } catch {
    return null
  }
}

/**
 * Resolve the native footer's ordered destinations.
 *
 * @param {Record<string, string | null | undefined> | null | undefined} [configured]
 *   Instance-level overrides keyed by the footer's internationalization key.
 * @returns {ReadonlyArray<{ key: string; href: string }>}
 */
export const resolveNativeFooterLinks = (configured) =>
  NATIVE_FOOTER_DESTINATION_KEYS.map((key) => ({
    key,
    href:
      normalizePlatformFooterDestination(configured?.[key]) ??
      DEFAULT_NATIVE_FOOTER_LINKS.find((link) => link.key === key)?.href ??
      PLATFORM_FOOTER_ORIGIN
  }))

/**
 * Read the native footer's instance-level overrides from the public
 * platform environment.
 *
 * @param {Record<string, string | undefined> | null | undefined} [environment]
 * @returns {Record<string, string | undefined>}
 */
export const readNativeFooterDestinationOverrides = (environment) => {
  const source = environment ?? {}
  return Object.fromEntries(
    NATIVE_FOOTER_DESTINATION_KEYS.map((key) => [
      key,
      source[NATIVE_FOOTER_DESTINATION_ENVIRONMENT_VARIABLES[key]]
    ])
  )
}
