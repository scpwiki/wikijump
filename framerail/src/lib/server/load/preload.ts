import defaults from "$lib/defaults"

import {
  limitLocalePreferences,
  parseAcceptLangHeader,
  uniqueLocales
} from "$lib/locales"

import { preloadView } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { buildPublicPreloadData } from "$lib/server/load/preload-data.js"
import { sanitizeUserData } from "$lib/server/load/user"

import type { PreloadData, Viewer } from "$lib/server/deepwell/views"
import type { Cookies } from "@sveltejs/kit"

const PAGE_ROUTES_WITH_ARTICLE_PRELOAD = new Set(["/", "/[slug]/[...extra]"])

export function pageRouteProvidesPreload(routeId: string | null) {
  return PAGE_ROUTES_WITH_ARTICLE_PRELOAD.has(routeId ?? "")
}

export function getPreloadRequestLocales(request: Request): string[] {
  return parseAcceptLangHeader(request)
}

export function getPreloadBackendLocales(locales: string[]): string[] {
  return limitLocalePreferences(
    [...locales, defaults.fallbackLocale],
    [defaults.fallbackLocale]
  )
}

export function finalizePreloadData(response: Viewer, locales: string[]): PreloadData {
  let resolvedLocales = uniqueLocales(locales)

  if (response.user_session?.user.locales) {
    resolvedLocales = uniqueLocales([
      ...limitLocalePreferences(response.user_session.user.locales),
      ...resolvedLocales
    ])
  }

  const requiredLocales = [response?.site?.locale, defaults.fallbackLocale].filter(
    (locale): locale is string => Boolean(locale)
  )
  resolvedLocales = limitLocalePreferences(
    [...resolvedLocales, ...requiredLocales],
    requiredLocales
  )

  const userSession = response.user_session
    ? { user: sanitizeUserData(response.user_session.user, false) }
    : null

  return buildPublicPreloadData(response, userSession, resolvedLocales)
}

/**
 * Loads common data that will be used in all routes, including site info,
 * user session and locales
 */
export async function loadPreload(request: Request, cookies: Cookies) {
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const locales = getPreloadRequestLocales(request)

  // Includes fallback locale in case there is no Accept-Language header
  const response = await preloadView(
    siteId,
    getPreloadBackendLocales(locales),
    sessionToken
  )

  return finalizePreloadData(response, locales)
}
