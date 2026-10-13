import defaults from "$lib/defaults"

import {
  limitLocalePreferences,
  parseAcceptLangHeader,
  uniqueLocales
} from "$lib/locales"

import { deleteSessionCookie } from "$lib/server/auth/session-cookie"
import { isInvalidSessionTokenError } from "$lib/server/deepwell/public-error.js"
import { preloadView } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { buildPublicPreloadData } from "$lib/server/load/preload-data.js"
import { sanitizeUserData } from "$lib/server/load/user"

import type { PreloadData, Viewer } from "$lib/server/deepwell/views"
import type { Cookies } from "@sveltejs/kit"

const PAGE_ROUTES_WITH_ARTICLE_PRELOAD = new Set([
  "/",
  "/[slug]/[...extra]",
  "/[x+2d]/user/[slug]"
])

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
  const acceptLocales = uniqueLocales(locales)
  const userLocales = response.user_session?.user.locales
    ? limitLocalePreferences(response.user_session.user.locales)
    : []

  const requiredLocales = [response?.site?.locale, defaults.fallbackLocale].filter(
    (locale): locale is string => Boolean(locale)
  )
  // Explicit user preferences win. Otherwise the site's own language defines
  // the page interface: current Wikidot serves e.g. scp-jp chrome in Japanese
  // regardless of the visitor's Accept-Language, so the site locale outranks
  // browser preferences, with the English fallback last.
  const resolvedLocales = limitLocalePreferences(
    [...userLocales, ...requiredLocales, ...acceptLocales],
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
  const backendLocales = getPreloadBackendLocales(locales)
  let response: Viewer

  try {
    response = await preloadView(siteId, backendLocales, sessionToken)
  } catch (error) {
    if (!sessionToken || !isInvalidSessionTokenError(error)) throw error

    deleteSessionCookie(cookies, request.url)
    response = await preloadView(siteId, backendLocales, undefined)
  }

  return finalizePreloadData(response, locales)
}
