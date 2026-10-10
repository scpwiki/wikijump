import defaults from "$lib/defaults"

import {
  buildWikidotPageActionLabels,
  wikidotPageActionVisibility
} from "$lib/wikidot/wikidot-page-actions"
import { buildWikidotPageInfoText } from "$lib/wikidot/wikidot-page-info"
import { buildWikidotPageWatchLabel } from "$lib/wikidot/wikidot-page-watch"
import { toIntlLocales } from "$lib/wikidot/wikidot-locale"

import type { PageView } from "$lib/server/deepwell/views"
import type { TranslateKeys } from "$lib/types"

interface PageTranslationContext {
  licenseName: string
  licenseUrl: string
  locales: string[]
  now?: number
}

interface WikidotPagePresentationContext {
  hasSession: boolean
  siteLocale: string
  now?: number
}

const basePageTranslateKeys = (
  licenseName: string,
  licenseUrl: string
): TranslateKeys => ({
  ...defaults.translateKeys,
  "save": {},
  "cancel": {},
  "title": {},
  "alt-title": {},
  "tags": {},
  "wiki-page-revision-comments": {},
  "wiki-page-append-content": {},
  "wiki-page-layout": {},
  "wiki-page-layout.default": {},
  "wiki-page-layout.wikidot": {},
  "wiki-page-layout.wikijump": {},
  "footer-license-unless": {
    license: licenseName,
    "license_url": licenseUrl
  }
})

const foundPageTranslateKeys = (
  response: Extract<PageView, { type: "found" }>,
  locales: string[],
  now: number
): TranslateKeys => {
  const updatedAt = Date.parse(
    response.data.page.updated_at ?? response.data.page.created_at
  )
  const daysDiff = Math.floor((now - updatedAt) / 1000 / 86400)

  return {
    "edit": {},
    "delete": {},
    "history": {},
    "move": {},
    "view": {},
    "vote": {},
    "layout": {},
    "parents": {},
    "options": {},
    "confirm": {},
    "wiki-page-revision": {
      revision: response.data.page_revision.revision_number
    },
    "wiki-page-last-edit": {
      date: new Date(updatedAt).toLocaleString(toIntlLocales(locales)),
      days: daysDiff
    },
    "wiki-page-revision-history": {},
    "wiki-page-revision-diff": {},
    "wiki-page-revision-diff.from": {},
    "wiki-page-revision-diff.to": {},
    "wiki-page-revision-diff.compare": {},
    "wiki-page-revision-diff.swap": {},
    "wiki-page-revision-diff.loading": {},
    "wiki-page-revision-diff.no-changes": {},
    "wiki-page-revision-number": {},
    "wiki-page-revision-created-at": {},
    "wiki-page-revision-user": {},
    "wiki-page-revision-rollback": {},
    "wiki-page-revision-type": {},
    "wiki-page-revision-type.create": {},
    "wiki-page-revision-type.regular": {},
    "wiki-page-revision-type.move": {},
    "wiki-page-revision-type.delete": {},
    "wiki-page-revision-type.rollback": {},
    "wiki-page-revision-type.undelete": {},
    "wiki-page-revision-type.undo": {},
    "wiki-page-vote": {},
    "wiki-page-vote.list": {},
    "wiki-page-vote.set": {},
    "wiki-page-vote.remove": {},
    "wiki-page-vote.score": {},
    "files": {},
    "upload": {},
    "restore": {},
    "wiki-page-file": {},
    "wiki-page-file-no-files": {},
    "wiki-page-file-upload.select": {},
    "wiki-page-file-upload.name": {},
    "wiki-page-file.name": {},
    "wiki-page-file.created-at": {},
    "wiki-page-file.updated-at": {},
    "wiki-page-file.mime": {},
    "wiki-page-file.size": {},
    "wiki-page-file.page": {},
    "wiki-page-file-move-destination-page": {},
    "wiki-page-file-revision-type": {},
    "wiki-page-file-revision-type.create": {},
    "wiki-page-file-revision-type.regular": {},
    "wiki-page-file-revision-type.move": {},
    "wiki-page-file-revision-type.delete": {},
    "wiki-page-file-revision-type.rollback": {},
    "wiki-page-file-revision-type.undelete": {},
    "wiki-page-file-revision-type.undo": {},
    "wiki-page-file-restore.new-page": {},
    "wiki-page-file-restore.new-name": {},
    "wiki-page-edit": {},
    "wiki-page-parent": {},
    "wiki-page-delete": {},
    "wiki-page-move": {},
    "wiki-page-move.new-slug": {},
    "wiki-page-no-render": {},
    "wiki-page-source": {},
    "wiki-page-view-source": {}
  }
}

const unavailablePageTranslateKeys = (): TranslateKeys => ({
  "restore": {},
  "wiki-page-restore": {},
  "wiki-page-restore.select": {},
  "wiki-page-create": {},
  "wiki-page-deleted": {
    datetime: "{$datetime}"
  }
})

export const buildPageTranslateKeys = (
  response: PageView,
  { licenseName, licenseUrl, locales, now = Date.now() }: PageTranslationContext
): TranslateKeys => ({
  ...basePageTranslateKeys(licenseName, licenseUrl),
  ...(response.type === "found"
    ? foundPageTranslateKeys(response, locales, now)
    : unavailablePageTranslateKeys())
})

export const buildWikidotPagePresentation = (
  response: PageView,
  { hasSession, siteLocale, now = Date.now() }: WikidotPagePresentationContext
) => {
  if (response.type !== "found") {
    return {
      wikidot_page_info: null,
      wikidot_page_actions: null,
      wikidot_page_watch: null
    }
  }

  const snapshot = response.data.wikidot_snapshot
  const imported = response.data.page.from_wikidot
  const revision = imported
    ? snapshot?.source_revision_count
    : response.data.page_revision.revision_number
  const updatedAt = imported
    ? snapshot?.source_updated_at
    : (response.data.page.updated_at ?? response.data.page.created_at)
  const wikidotPageInfo =
    revision !== undefined && updatedAt
      ? buildWikidotPageInfoText({
          revision,
          updatedAt,
          now,
          locale: siteLocale
        })
      : null

  if (!imported) {
    return {
      wikidot_page_info: wikidotPageInfo,
      wikidot_page_actions: null,
      wikidot_page_watch: null
    }
  }

  const { showRate, showDiscuss } = wikidotPageActionVisibility({
    sourceSite: snapshot?.source_site,
    ratingEnabled: response.data.page_rating.enabled,
    discussionEnabled: response.data.page_discussion.enabled,
    discussionThreadId: response.data.page.discussion_thread_id
  })

  return {
    wikidot_page_info: wikidotPageInfo,
    wikidot_page_actions: buildWikidotPageActionLabels({
      rating: snapshot?.imported_rating ?? null,
      comments: snapshot?.comments ?? null,
      locale: siteLocale,
      showRate,
      showDiscuss
    }),
    wikidot_page_watch: buildWikidotPageWatchLabel({
      sourceSite: snapshot?.source_site,
      hasSession,
      locale: siteLocale
    })
  }
}
