import { fixtureState, hasExactKeys, requestContextHeaders } from "./context.js"
import { pages, toArticleViewResult } from "./data.js"

const LISTPAGES_NAVIGATION_EXTRA = /^p\/[1-9][0-9]*$/u

const pageForArticleRoute = (route) => {
  const page = pages[route.slug]
  if (!page) return null
  if (route.slug === "listpages-navigation") {
    if (route.extra !== "" && !LISTPAGES_NAVIGATION_EXTRA.test(route.extra)) {
      return null
    }
    return {
      ...page,
      compiled_body_html: [
        `<span id="listpages-route">${route.extra || "root"}</span>`,
        '<div class="pager">',
        '<span class="target"><a id="listpages-page-one" href="/listpages-navigation/p/1">1</a></span>',
        '<span class="target"><a id="listpages-page-two" href="/listpages-navigation/p/2">2</a></span>',
        "</div>"
      ].join("")
    }
  }
  return route.extra === "" ? page : null
}

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleArticleRpc = ({ rpcRequest, request }) => {
  const { articleReadRequests, pageReadRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "article_view" &&
    ((hasExactKeys(rpcRequest.params, ["locales", "route", "site_id"]) &&
      rpcRequest.params.session_token === undefined) ||
      (hasExactKeys(rpcRequest.params, [
        "locales",
        "route",
        "session_token",
        "site_id"
      ]) &&
        rpcRequest.params.session_token === "fixture-session-token")) &&
    rpcRequest.params.site_id === 6000005 &&
    Array.isArray(rpcRequest.params.locales) &&
    hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
    typeof rpcRequest.params.route.slug === "string" &&
    pageForArticleRoute(rpcRequest.params.route)
  ) {
    articleReadRequests.articleView.push(rpcRequest.params)
    result = toArticleViewResult(pageForArticleRoute(rpcRequest.params.route))
  } else if (
    rpcRequest.method === "article_view_cache_metadata" &&
    hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    rpcRequest.params.session_token === null &&
    Array.isArray(rpcRequest.params.locales) &&
    hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
    typeof rpcRequest.params.route.slug === "string" &&
    pageForArticleRoute(rpcRequest.params.route)
  ) {
    articleReadRequests.articleViewCacheMetadata.push(rpcRequest.params)
    const page = pageForArticleRoute(rpcRequest.params.route)
    result = {
      article_page_cache_key: `deepwell:article-view:page:v1:site=6000005:page=${page.page_id}:rev=${page.revision_id}:updated=0:permission=site=0,user=0:body=fixture`,
      public_content_cache_fence: "0",
      anonymous_permission_cache_fence: "site=0,user=0"
    }
  } else if (
    rpcRequest.method === "page_view" &&
    hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    Array.isArray(rpcRequest.params.locales) &&
    rpcRequest.params.session_token === "fixture-session-token" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
    typeof rpcRequest.params.route.slug === "string" &&
    rpcRequest.params.route.extra === ""
  ) {
    pageReadRequests.pageView.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const page = pages[rpcRequest.params.route.slug]
    result = page
      ? page.slug === "private-page"
        ? { type: "forbidden", data: {} }
        : { type: "found", data: { page: { slug: page.slug } } }
      : { type: "missing", data: {} }
  } else {
    return undefined
  }

  return { result }
}
