import defaults from "$lib/defaults"

import { parseAcceptLangHeader } from "$lib/locales"

import { preloadView } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { sanitizeUserData } from "$lib/server/load/user"

import type { Viewer } from "$lib/server/deepwell/views"
import type { Cookies } from "@sveltejs/kit"

export function getPreloadRequestLocales(request: Request): string[] {
  return parseAcceptLangHeader(request)
}

export function getPreloadBackendLocales(locales: string[]): string[] {
  return [...locales, defaults.fallbackLocale]
}

export function finalizePreloadData(response: Viewer, locales: string[]) {
  let resolvedLocales = [...locales]

  if (response.user_session?.user.locales) {
    resolvedLocales = [
      ...response.user_session.user.locales,
      ...resolvedLocales.filter(
        (locale) => !response.user_session?.user.locales.includes(locale)
      )
    ]
  }

  if (response?.site?.locale && !resolvedLocales.includes(response.site.locale)) {
    resolvedLocales.push(response.site.locale)
  }

  if (!resolvedLocales.includes(defaults.fallbackLocale)) {
    resolvedLocales.push(defaults.fallbackLocale)
  }

  if (response.user_session?.user) {
    response.user_session.user = sanitizeUserData(response.user_session?.user, false)
  }

  return { ...response, locales: resolvedLocales }
}

/**
 * Loads common data that will be used in all routes, including site info,
 * user session and locales
 */
export async function loadPreload(request: Request, cookies: Cookies) {
  // Set up parameters
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const locales = getPreloadRequestLocales(request)

  // Request data from backend
  // Includes fallback locale in case there is no Accept-Language header
  const response = await preloadView(
    siteId,
    getPreloadBackendLocales(locales),
    sessionToken
  )

  // Handover data to subsequent requests for rendering
  return finalizePreloadData(response, locales)
}
