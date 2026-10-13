import { handleAjaxModuleConnectorRequest } from "$lib/server/ajax-module-connector.js"
import {
  renderWikidotEditMeta,
  renderWikidotPageFiles,
  renderWikidotPageRevisionList,
  renderWikidotPageRevisionSource,
  renderWikidotPageRevisionVersion,
  renderWikidotWhoRated,
  renderWikidotWhoRatedUnavailable,
  renderWikidotViewSource
} from "$lib/server/ajax-module-connector-page-reads.js"
import { authGetSession } from "$lib/server/auth/get-session"
import { client } from "$lib/server/deepwell"
import { siteEducationalUpgrade } from "$lib/server/deepwell/admin"
import { forumPostCreate, wikidotForumModule } from "$lib/server/deepwell/forum"
import { wikidotMembersListModule } from "$lib/server/deepwell/membership"
import { adminView, preloadView } from "$lib/server/deepwell/views"
import { pageFileList } from "$lib/server/deepwell/page-file"
import {
  pageDelete,
  pageDraftExists,
  pageDraftRemove,
  pageDraftSave,
  pageEdit,
  pageGet,
  pageHistory,
  pageMetaTagDelete,
  pageMetaTagSet,
  pageMetaTags,
  pageRevision,
  pageRevisionById,
  pageScore,
  pageViewPermission,
  pageWhoRated,
  pageParentUpdate,
  siteToolsListDrafts,
  siteToolsOrphanedPages,
  siteToolsWantedPages,
  wikidotPageDiscussionCreate,
  wikidotSiteChangesModule
} from "$lib/server/deepwell/page"
import {
  renderWikidotListDrafts,
  renderWikidotOrphanedPages,
  renderWikidotSiteTools,
  renderWikidotWantedPages
} from "$lib/server/wikidot-site-tools.js"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { renderWikidotManageSiteGeneral } from "$lib/server/wikidot-manage-site-general.js"
import { renderWikidotManageSiteEducational } from "$lib/server/wikidot-manage-site-educational.js"

import type { RequestHandler } from "./$types"

interface WikidotListPagesModuleOutput {
  body: string
}

interface WikidotCategoriesPageListModuleOutput {
  body: string
}

const pageSlugFromReferer = (request: Request): string | undefined => {
  const referer = request.headers.get("referer")
  if (!referer) return undefined

  try {
    const refererUrl = new URL(referer)
    const requestUrl = new URL(request.url)
    if (refererUrl.origin !== requestUrl.origin) return undefined
    const [slug] = refererUrl.pathname.split("/").filter(Boolean)
    return slug ? decodeURIComponent(slug) : undefined
  } catch {
    return undefined
  }
}

export const POST: RequestHandler = async ({ request, cookies, getClientAddress }) => {
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const sourcePage = pageSlugFromReferer(request)
  const requestContext = {
    sessionToken,
    siteId,
    page: sourcePage
  }
  let userIdPromise: Promise<number | undefined> | undefined
  const resolveNewPageUserId = () => {
    userIdPromise ??= (async () => {
      const session = sessionToken ? await authGetSession(sessionToken) : undefined
      return session?.user_id
    })()
    return userIdPromise
  }

  return handleAjaxModuleConnectorRequest(request, {
    siteId,
    canCreateNewPage: async () => (await resolveNewPageUserId()) !== undefined,
    pageExists: (slug: string) =>
      pageViewPermission(siteId, slug, {
        ...requestContext,
        page: slug
      }),
    createNewPage: async ({
      slug,
      title,
      wikitext,
      tags,
      parentPage
    }: {
      slug: string
      title: string
      wikitext: string
      tags: string[]
      parentPage: string
    }) => {
      const userId = await resolveNewPageUserId()
      if (userId === undefined) {
        throw new Error("NewPage autosave requires a page mutation actor")
      }
      const created = await pageEdit(
        {
          siteId,
          pageId: undefined,
          userId,
          userIpAddr: getClientAddress(),
          slug,
          lastRevisionId: undefined,
          revisionComments: "",
          wikitext,
          title,
          altTitle: "",
          tags,
          layout: null
        },
        requestContext
      )
      if (parentPage && created.page_id !== undefined) {
        await pageParentUpdate(siteId, slug, userId, [parentPage], undefined, {
          ...requestContext,
          page: slug
        })
      }
    },
    createPageDiscussion: ({ pageId }: { siteId: number; pageId: number }) =>
      wikidotPageDiscussionCreate(siteId, pageId, {
        ...requestContext,
        page: pageId
      }),
    createForumPost: (input) => forumPostCreate(input, requestContext),
    renderEditMetaModule: async ({
      siteId: requestSiteId,
      pageId
    }: {
      siteId: number
      pageId: number
    }) => {
      const tags = await pageMetaTags(requestSiteId, pageId, {
        ...requestContext,
        page: pageId
      })
      return {
        status: "ok",
        body: renderWikidotEditMeta(tags),
        js_include: [
          "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/edit/EditMetaModule.js"
        ]
      }
    },
    saveMetaTag: ({
      siteId: requestSiteId,
      pageId,
      name,
      content,
      allPages
    }: {
      siteId: number
      pageId: number
      name: string
      content: string
      allPages: boolean
    }) =>
      pageMetaTagSet(requestSiteId, pageId, name, content, allPages, {
        ...requestContext,
        page: pageId
      }),
    deleteMetaTag: ({
      siteId: requestSiteId,
      pageId,
      name,
      allPages
    }: {
      siteId: number
      pageId: number
      name: string
      allPages: boolean
    }) =>
      pageMetaTagDelete(requestSiteId, pageId, name, allPages, {
        ...requestContext,
        page: pageId
      }),
    deletePage: async ({
      siteId: requestSiteId,
      pageId
    }: {
      siteId: number
      pageId: number
    }) => {
      const userId = await resolveNewPageUserId()
      if (userId === undefined) {
        throw new Error("deletePage requires a page mutation actor")
      }
      const page = (await pageGet(requestSiteId, pageId, {
        ...requestContext,
        page: pageId
      })) as { slug?: string; revision_id?: number } | null
      const revisionId = page?.revision_id
      if (
        page === null ||
        typeof page.slug !== "string" ||
        revisionId === undefined ||
        !Number.isSafeInteger(revisionId)
      ) {
        throw new Error("deletePage target is unavailable")
      }
      await pageDelete(
        {
          siteId: requestSiteId,
          pageId,
          userId,
          userIpAddr: getClientAddress(),
          slug: page.slug,
          lastRevisionId: revisionId,
          revisionComments: ""
        },
        {
          ...requestContext,
          page: pageId
        }
      )
    },
    savePageDraft: async ({
      siteId: requestSiteId,
      pageId,
      slug,
      title,
      wikitext
    }: {
      siteId: number
      pageId?: number
      slug: string
      title: string
      wikitext: string
    }) => {
      const userId = await resolveNewPageUserId()
      if (userId === undefined) throw new Error("page draft requires a mutation actor")
      await pageDraftSave(
        { siteId: requestSiteId, userId, pageId, slug, title, wikitext },
        { ...requestContext, page: slug }
      )
    },
    pageDraftExists: async ({
      siteId: requestSiteId,
      pageId,
      slug
    }: {
      siteId: number
      pageId?: number
      slug: string
    }) => {
      const userId = await resolveNewPageUserId()
      if (userId === undefined) return false
      return pageDraftExists(
        { siteId: requestSiteId, userId, pageId, slug },
        { ...requestContext, page: slug }
      )
    },
    removePageDraft: async ({
      siteId: requestSiteId,
      pageId,
      slug
    }: {
      siteId: number
      pageId?: number
      slug: string
    }) => {
      const userId = await resolveNewPageUserId()
      if (userId === undefined) {
        throw new Error("page draft removal requires a mutation actor")
      }
      await pageDraftRemove(
        { siteId: requestSiteId, userId, pageId, slug },
        { ...requestContext, page: slug }
      )
    },
    renderListPages: ({
      siteId,
      moduleBody,
      parameters
    }: {
      siteId: number
      moduleBody: string
      parameters: Record<string, string>
    }) =>
      Promise.resolve(
        client.request(
          "wikidot_list_pages_module",
          {
            site_id: siteId,
            module_body: moduleBody,
            parameters
          },
          { siteId }
        )
      ) as Promise<WikidotListPagesModuleOutput>,
    renderForumModule: ({
      siteId,
      moduleName,
      parameters
    }: {
      siteId: number
      moduleName: string
      parameters: Record<string, string>
    }) => wikidotForumModule(siteId, moduleName, parameters, requestContext),
    renderSiteChangesModule: ({
      siteId,
      pageId,
      page,
      perpage,
      categoryId,
      options
    }: {
      siteId: number
      pageId?: string
      page: string
      perpage: string
      categoryId?: string
      options: string
    }) =>
      wikidotSiteChangesModule(
        siteId,
        { pageId, page, perpage, categoryId, options },
        requestContext
      ),
    renderMembersList: ({
      siteId,
      parameters
    }: {
      siteId: number
      parameters: Record<string, string>
    }) => wikidotMembersListModule(siteId, parameters, requestContext),
    renderCategoriesPageList: ({
      siteId,
      categoryId
    }: {
      siteId: number
      categoryId: number
    }) =>
      Promise.resolve(
        client.request(
          "wikidot_categories_page_list_module",
          {
            site_id: siteId,
            category_id: categoryId
          },
          requestContext
        )
      ).then((output) => ({
        status: "ok",
        body: (output as WikidotCategoriesPageListModuleOutput).body
      })),
    renderManageSiteGeneralModule: async ({ siteId }: { siteId: number }) => {
      const locales = getPreloadBackendLocales(getPreloadRequestLocales(request))
      const authorization = await adminView(siteId, locales, sessionToken)
      if (authorization.type !== "site_found") return null

      const preload = await preloadView(siteId, locales, sessionToken)
      return {
        status: "ok",
        body: renderWikidotManageSiteGeneral(preload.site),
        js_include: [
          "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/managesite/ManageSiteGeneralModule.js"
        ]
      }
    },
    renderManageSiteEducationalModule: async ({ siteId }: { siteId: number }) => {
      const locales = getPreloadBackendLocales(getPreloadRequestLocales(request))
      const authorization = await adminView(siteId, locales, sessionToken)
      if (authorization.type !== "site_found" || !authorization.data.is_master_admin) {
        return null
      }

      const preload = await preloadView(siteId, locales, sessionToken)
      if (preload.site.educational) return null
      return {
        status: "ok",
        body: renderWikidotManageSiteEducational(),
        js_include: [
          "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/managesite/ManageSiteUpgradeEduModule.js"
        ]
      }
    },
    upgradeEducationalSite: async ({
      siteId,
      organization,
      purpose
    }: {
      siteId: number
      organization: string
      purpose: string
    }) => {
      if (!sessionToken) throw new Error("Educational upgrade requires a session")
      const session = await authGetSession(sessionToken)
      if (!session) throw new Error("Educational upgrade requires a valid session")
      const locales = getPreloadBackendLocales(getPreloadRequestLocales(request))
      const preload = await preloadView(siteId, locales, sessionToken)
      await siteEducationalUpgrade(
        siteId,
        preload.site.settings_revision,
        session.user_id,
        getClientAddress(),
        organization,
        purpose,
        { sessionToken, siteId }
      )
    },
    renderSiteToolsModule: async ({
      siteId,
      moduleName
    }: {
      siteId: number
      moduleName: string
      parameters: Record<string, string>
    }) => {
      if (moduleName === "sitetools/SiteToolsModule") {
        return { status: "ok", body: renderWikidotSiteTools() }
      }
      if (moduleName === "sitetools/WantedPagesModule") {
        const targets = await siteToolsWantedPages(siteId, requestContext)
        return { status: "ok", body: renderWikidotWantedPages(targets) }
      }
      if (moduleName === "sitetools/OrphanedPagesModule") {
        const pages = await siteToolsOrphanedPages(siteId, requestContext)
        return { status: "ok", body: renderWikidotOrphanedPages(pages) }
      }
      if (moduleName === "list/ListDraftsModule") {
        const drafts = await siteToolsListDrafts(siteId, requestContext)
        return { status: "ok", body: renderWikidotListDrafts(drafts) }
      }
      return { status: "not_ok", body: "" }
    },
    renderPageReadModule: async ({
      siteId,
      moduleName,
      parameters
    }: {
      siteId: number
      moduleName: string
      parameters: Record<string, string>
    }) => {
      if (moduleName === "pagerate/WhoRatedPageModule") {
        const pageId = Number.parseInt(parameters.pageId, 10)
        const votes = await pageWhoRated(siteId, pageId, {
          ...requestContext,
          page: pageId
        })
        if (votes.length === 0 && Number.isSafeInteger(pageId)) {
          // An imported page can carry a rating baseline without any
          // imported per-voter rows; the pinned empty roster would falsely
          // claim nobody voted (issue #2205). The combined score is nonzero
          // only through that baseline when no current vote rows exist, so
          // declare the list unavailable instead. Any lookup failure falls
          // through to the pinned empty body.
          try {
            const { score } = await pageScore({
              ...requestContext,
              page: pageId
            })
            if (score !== 0) {
              return { status: "ok", body: renderWikidotWhoRatedUnavailable() }
            }
          } catch {
            // Fall through to the pinned empty roster below.
          }
        }
        return { status: "ok", body: renderWikidotWhoRated(votes) }
      }
      if (
        moduleName === "history/PageSourceModule" ||
        moduleName === "history/PageVersionModule"
      ) {
        const revisionId = Number.parseInt(parameters.revision_id, 10)
        const revision = await pageRevisionById(
          siteId,
          revisionId,
          moduleName === "history/PageVersionModule",
          moduleName === "history/PageSourceModule",
          requestContext
        )
        if (!revision) throw new Error("History target revision is missing")
        return {
          status: "ok",
          body:
            moduleName === "history/PageSourceModule"
              ? renderWikidotPageRevisionSource(revision)
              : renderWikidotPageRevisionVersion(revision)
        }
      }

      const pageId = Number.parseInt(parameters.page_id, 10)
      const pageRequestContext = { ...requestContext, page: pageId }
      if (moduleName === "history/PageRevisionListModule") {
        const revisions = await pageHistory(
          siteId,
          pageId,
          -1,
          Number.parseInt(parameters.perpage, 10),
          pageRequestContext,
          true
        )
        const locales = getPreloadBackendLocales(getPreloadRequestLocales(request))
        const { site } = await preloadView(siteId, locales, sessionToken)
        return {
          status: "ok",
          body: renderWikidotPageRevisionList(revisions, site.locale)
        }
      }
      if (moduleName === "viewsource/ViewSourceModule") {
        const revision = await pageRevision(
          siteId,
          pageId,
          undefined,
          false,
          true,
          pageRequestContext
        )
        if (!revision) throw new Error("ViewSource target page is missing")
        return {
          status: "ok",
          body: renderWikidotViewSource(revision.wikitext ?? "")
        }
      }

      const [page, files] = await Promise.all([
        pageGet(siteId, pageId, pageRequestContext) as Promise<{ slug: string } | null>,
        pageFileList(siteId, pageId, false, pageRequestContext)
      ])
      if (!page) throw new Error("PageFiles target page is missing")
      return {
        status: "ok",
        body: renderWikidotPageFiles(page.slug, files)
      }
    }
  })
}
