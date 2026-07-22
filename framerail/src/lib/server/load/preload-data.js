/**
 * Project Deepwell viewer data onto the explicit browser-visible preload
 * DTO. New internal fields added to the backend response stay server-only
 * unless they are deliberately added here.
 *
 * @template Site
 * @template UserSession
 * @param {{
 *   site: Site
 *   site_file_domain: string
 *   license_name: string
 *   license_url: string
 *   license_kind: "standard" | "other" | "copyright"
 *   license_html: string | null
 * }} response
 * @param {UserSession} userSession
 * @param {string[]} locales
 */
export const buildPublicPreloadData = (response, userSession, locales) => ({
  site: response.site,
  site_file_domain: response.site_file_domain,
  license_name: response.license_name,
  license_url: response.license_url,
  license_kind: response.license_kind,
  license_html: response.license_html,
  user_session: userSession,
  locales
})
