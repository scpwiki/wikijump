import type { DataFormEditor, PageTemplateSummary } from "$lib/server/deepwell/views"
import type { buildPageErrorForms } from "$lib/server/load/page/page-forms"
import type { PageOptions } from "$lib/types"

export type PageErrorData = {
  view: "missing" | "permissions"
  forms: Awaited<ReturnType<typeof buildPageErrorForms>>
  page_templates?: PageTemplateSummary[]
  selected_template_page_id?: number | null
  data_form?: DataFormEditor | null
  can_create?: boolean
  site: {
    site_id: number
    default_page: string
  }
  options: PageOptions
  compiled_body_html: string
  internationalization?: Record<string, string>
}

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value)

const isSuperValidatedForm = (value: unknown) =>
  isRecord(value) &&
  typeof value.valid === "boolean" &&
  typeof value.posted === "boolean" &&
  isRecord(value.data) &&
  isRecord(value.errors) &&
  isRecord(value.constraints)

export function isPageErrorData(value: unknown): value is PageErrorData {
  if (!isRecord(value)) return false
  if (value.view !== "missing" && value.view !== "permissions") return false
  if (!isRecord(value.forms)) return false
  if (
    !isSuperValidatedForm(value.forms.pageEditForm) ||
    !isSuperValidatedForm(value.forms.pageRestoreForm)
  ) {
    return false
  }
  if (!isRecord(value.site)) return false
  if (!Number.isSafeInteger(value.site.site_id)) return false
  if (typeof value.site.default_page !== "string") return false
  if (!isRecord(value.options)) return false
  if (
    typeof value.options.edit !== "boolean" ||
    typeof value.options.no_redirect !== "boolean" ||
    typeof value.options.no_render !== "boolean" ||
    typeof value.options.debug !== "boolean" ||
    typeof value.options.renderer !== "boolean" ||
    typeof value.options.comments !== "boolean" ||
    typeof value.options.history !== "boolean" ||
    typeof value.options.data !== "string"
  ) {
    return false
  }
  if (typeof value.compiled_body_html !== "string") return false
  if (value.internationalization !== undefined && !isRecord(value.internationalization)) {
    return false
  }
  if (
    value.page_templates !== undefined &&
    (!Array.isArray(value.page_templates) ||
      !value.page_templates.every(
        (template) =>
          isRecord(template) &&
          Number.isSafeInteger(template.page_id) &&
          typeof template.slug === "string" &&
          typeof template.title === "string" &&
          typeof template.wikitext === "string"
      ))
  ) {
    return false
  }
  return true
}
