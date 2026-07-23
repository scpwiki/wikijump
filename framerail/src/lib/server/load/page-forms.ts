import {
  layoutSchema,
  pageDeleteSchema,
  pageEditSchema,
  pageMoveSchema
} from "$lib/server/load/page-edit-actions"
import {
  pageFileEditSchema,
  pageFileMoveSchema,
  pageFileRestoreSchema,
  pageFileUploadSchema
} from "$lib/server/load/page-file-actions"
import { pageParentSchema } from "$lib/server/load/page-relation-actions"
import { pageRestoreSchema } from "$lib/server/load/page-revision-actions"
import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"

import type { PageView } from "$lib/server/deepwell/views"

export const buildPageForms = async (request: Request) => ({
  pageDeleteForm: await superValidate(request, valibot(pageDeleteSchema)),
  pageEditForm: await superValidate(request, valibot(pageEditSchema)),
  fileUploadForm: await superValidate(request, valibot(pageFileUploadSchema)),
  fileEditForm: await superValidate(request, valibot(pageFileEditSchema)),
  fileMoveForm: await superValidate(request, valibot(pageFileMoveSchema)),
  fileRestoreForm: await superValidate(request, valibot(pageFileRestoreSchema)),
  layoutForm: await superValidate(request, valibot(layoutSchema)),
  pageMoveForm: await superValidate(request, valibot(pageMoveSchema)),
  pageParentForm: await superValidate(request, valibot(pageParentSchema)),
  pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
})

export const buildPageErrorForms = async (request: Request, response: PageView) => {
  const pageEditForm = await superValidate(request, valibot(pageEditSchema))
  if (response.type === "missing" && response.data.new_page_wikitext !== null) {
    pageEditForm.data.wikitext = response.data.new_page_wikitext
  }

  return {
    pageEditForm,
    pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
  }
}
