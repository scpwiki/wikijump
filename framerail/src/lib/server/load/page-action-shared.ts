import { normalizeActionError } from "$lib/server/load/action-error"
import { fail } from "@sveltejs/kit"
import { number } from "valibot"

const DEEPWELL_PERMISSION_DENIED = 3106

export const pageMutationBaseSchema = {
  pageId: number(),
  siteId: number(),
  lastRevisionId: number()
}

export function failForActionError(error: unknown, body: Record<string, unknown> = {}) {
  const details = normalizeActionError(error)
  return fail(details.code === DEEPWELL_PERMISSION_DENIED ? 403 : 500, {
    ...body,
    ...details
  })
}

export function failForMissingSession(body: Record<string, unknown> = {}) {
  return fail(401, {
    ...body,
    message: "Authentication required."
  })
}
