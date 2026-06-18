import defaults from "$lib/defaults"

import { translate } from "$lib/server/deepwell/translate"
import { error } from "@sveltejs/kit"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import type { TranslateKeys } from "$lib/types"

type PreloadData = Awaited<ReturnType<PreloadDataAsync>>

async function loadForumLabels(parentData: PreloadData, translateKeys: TranslateKeys) {
  const internationalization = await translate(parentData.locales, {
    ...defaults.translateKeys,
    ...translateKeys
  })

  return { internationalization }
}

function validateForumRouteId(routeId: number) {
  if (Number.isNaN(routeId)) {
    error(404)
  }
}

export function parseForumRouteId(routeId: string) {
  const parsedRouteId = Number.parseInt(routeId, 10)
  validateForumRouteId(parsedRouteId)
  return parsedRouteId
}

export async function loadForumIndex(parentData: PreloadData) {
  return loadForumLabels(parentData, {
    forum: {},
    "forum-route.index": {}
  })
}

export async function loadForumFallback(parentData: PreloadData) {
  return loadForumLabels(parentData, {
    "forum-route.invalid": {}
  })
}

export async function loadForumCategory(categoryId: number, parentData: PreloadData) {
  validateForumRouteId(categoryId)

  return {
    categoryId,
    ...(await loadForumLabels(parentData, {
      "forum-category": {},
      "forum-category.loaded": { categoryId }
    }))
  }
}

export async function loadForumThread(threadId: number, parentData: PreloadData) {
  validateForumRouteId(threadId)

  return {
    threadId,
    ...(await loadForumLabels(parentData, {
      "forum-thread": {},
      "forum-thread.loaded": { threadId }
    }))
  }
}
