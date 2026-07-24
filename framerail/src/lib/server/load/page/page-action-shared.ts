import { number } from "valibot"

export {
  failForActionError,
  failForMissingSession,
  readActionJson
} from "$lib/server/load/action-error"

export const pageMutationBaseSchema = {
  pageId: number(),
  siteId: number(),
  lastRevisionId: number()
}

export const pageActionBaseSchema = {
  pageId: number(),
  siteId: number()
}
