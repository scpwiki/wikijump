import defaults from "$lib/defaults"
import { parseAcceptLangHeader } from "$lib/locales"
import { translate } from "$lib/server/deepwell/translate"
import { pageView } from "$lib/server/deepwell/views"
import type { Optional, TranslateKeys } from "$lib/types"
import { error, redirect } from "@sveltejs/kit"

// TODO form single deepwell request that does all the relevant prep stuff here

export async function loadPage(
  slug: Optional<string>,
  extra: Optional<string>,
  request,
  cookies
) {
  // Set up parameters
  const url = new URL(request.url)
  const domain = url.hostname
  const route = slug || extra ? { slug, extra } : null
  const sessionToken = cookies.get("wikijump_token")
  let locales = parseAcceptLangHeader(request)

  // TODO insert user preference at the beginning of the list

  // Request data from backend
  // Includes fallback locale in case there is no Accept-Language header
  const response = await pageView(
    domain,
    [...locales, defaults.fallbackLocale],
    route,
    sessionToken
  )

  if (response.data?.site?.locale && !locales.includes(response.data.site.locale)) {
    locales.push(response.data.site.locale)
  }

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  // Process response, performing redirects etc
  const viewData = response.data
  viewData.view = response.type

  let checkRedirect = true
  let errorStatus = null

  switch (response.type) {
    case "page_found":
      break
    case "page_missing":
      viewData.page = null
      viewData.page_revision = null
      errorStatus = 404
      break
    case "page_permissions":
      errorStatus = 403
      break
    case "site_missing":
      checkRedirect = false
      errorStatus = 404
  }

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys,

    // Page actions
    "save": {},
    "cancel": {},

    // Page edit
    "title": {},
    "alt-title": {},
    "tags": {},
    "wiki-page-revision-comments": {},
    "wiki-page-layout-default": {},
    "wiki-page-layout-wikidot": {},
    "wiki-page-layout-wikijump": {}
  }

  if (errorStatus === null) {
    translateKeys = {
      ...translateKeys,

      // Page actions
      "edit": {},
      "delete": {},
      "history": {},
      "move": {},
      "view": {},
      "vote": {},
      "layout": {},
      "parents": {},

      // Page history
      "wiki-page-revision": {
        revision: viewData.page_revision.revision_number
      },
      "wiki-page-revision-number": {},
      "wiki-page-revision-created-at": {},
      "wiki-page-revision-user": {},
      "wiki-page-revision-rollback": {},

      // Page vote
      "wiki-page-vote-list": {},
      "wiki-page-vote-set": {},
      "wiki-page-vote-remove": {},
      "wiki-page-vote-score": {},

      // Misc
      "wiki-page-move-new-slug": {},
      "wiki-page-no-render": {},
      "wiki-page-view-source": {}
    }
  }

  const translated = await translate(locales, translateKeys)

  viewData.internationalization = translated

  if (errorStatus !== null) {
    throw error(errorStatus, viewData)
  }

  // TODO remove checkRedirect when errorStatus is fixed
  if (checkRedirect) {
    runRedirect(viewData, domain, slug, extra)
  }

  // Return to page for rendering
  return viewData
}

function runRedirect(
  viewData,
  originalDomain: string,
  originalSlug: Optional<string>,
  extra: Optional<string>
): void {
  if (!viewData.redirectSite && !viewData.redirectPage) {
    // Nothing to do
    return
  }

  const domain: string = viewData.redirectSite || originalDomain
  const slug: Optional<string> = viewData.redirectPage || originalSlug
  const route: string = buildRoute(slug, extra)
  throw redirect(308, `https://${domain}/${route}`)
}

function buildRoute(slug: Optional<string>, extra: Optional<string>): string {
  // Combines a nullable slug and extra to form a route for redirection.
  //
  // Test cases:
  // null, null => ''
  // 'start', null => 'start'
  // 'start', '' => 'start'
  // 'start', 'comments/show' => 'start/comments/show'
  // null, 'xyz' => (impossible)

  if (slug === null) {
    return ""
  } else if (!extra) {
    return slug
  } else {
    return `${slug}/${extra}`
  }
}
