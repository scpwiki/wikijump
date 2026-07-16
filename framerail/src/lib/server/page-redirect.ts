export interface PageRedirectData {
  redirect_page: string | null
  redirect_kind?: "wikidot_module" | null
}

export interface ResolvedPageRedirect {
  status: 301 | 308
  location: string
}

export function resolvePageRedirect(
  viewData: PageRedirectData,
  originalSlug: string | null | undefined,
  extra: string | null | undefined,
  requestUrl: string
): ResolvedPageRedirect | null {
  if (!viewData.redirect_page) {
    return null
  }

  if (viewData.redirect_kind === "wikidot_module") {
    return resolveWikidotModuleRedirect(viewData.redirect_page, requestUrl)
  }
  if (viewData.redirect_kind !== null && viewData.redirect_kind !== undefined) {
    return null
  }

  return {
    status: 308,
    location: `/${buildRoute(viewData.redirect_page || originalSlug, extra)}`
  }
}

function resolveWikidotModuleRedirect(
  location: string,
  requestUrl: string
): ResolvedPageRedirect | null {
  if (
    location.startsWith("//") ||
    (!location.startsWith("/") && !/^https?:\/\//i.test(location))
  ) {
    return null
  }

  let current: URL
  let target: URL
  try {
    current = new URL(requestUrl)
    target = new URL(location, current)
  } catch {
    return null
  }

  if (
    !["http:", "https:"].includes(target.protocol) ||
    target.username !== "" ||
    target.password !== ""
  ) {
    return null
  }

  let currentPath: string
  let targetPath: string
  try {
    currentPath = decodeURI(current.pathname)
    targetPath = decodeURI(target.pathname)
  } catch {
    return null
  }

  // Fragments never reach the server. Hosts are compared without protocol or
  // port so an HTTP downgrade cannot bounce through the canonical HTTPS route.
  if (
    target.hostname === current.hostname &&
    targetPath === currentPath &&
    target.search === current.search
  ) {
    return null
  }

  return { status: 301, location }
}

function buildRoute(
  slug: string | null | undefined,
  extra: string | null | undefined
): string {
  if (slug === null) {
    return ""
  } else if (!extra) {
    return slug ?? ""
  } else {
    return `${slug}/${extra}`
  }
}
