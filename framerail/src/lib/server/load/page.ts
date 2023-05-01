import { pageView } from "$lib/server/deepwell/views.ts"
import type { Optional } from "$lib/types.ts"
import { redirect } from "@sveltejs/kit"

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
  const language = request.headers.get("Accept-Language")

  // TODO set up svelte i18n, see WJ-1175
  //
  // TODO also set up deepwell fluent so that fallback
  //      languages are used, i.e. if I do en-GB it falls back to
  //      en generic
  const locale = "en"

  // Request data from backend
  const response = await pageView(domain, locale, route, sessionToken)

  // Process response, performing redirects etc
  const viewData = response.data
  viewData.view = response.type

  let checkRedirect = true
  let status = null

  switch (response.type) {
    case "pageFound":
      break
    case "pageMissing":
      viewData.page = null
      viewData.pageRevision = null
      status = 404
      break
    case "pagePermissions":
      status = 403
      break
    case "siteMissing":
      checkRedirect = false
      status = 404
  }

  if (checkRedirect) {
    runRedirect(viewData, domain, slug, extra)
  }

  // Return to page for rendering
  // TODO make page view into a component
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
