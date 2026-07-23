import defaults from "$lib/defaults"
import { discussionUpdateValue } from "$lib/admin-forum.js"
import { licenseUpdateValue } from "$lib/admin-license.js"
import { navigationUpdateValues } from "$lib/admin-navigation.js"

import { authGetSession } from "$lib/server/auth/getSession"
import {
  categoryLicenseUpdate,
  categoryNavigationUpdate,
  categoryDiscussionUpdate,
  categoryRatingUpdate,
  categoryTemplateUpdate,
  siteForumNestingUpdate,
  siteIconsUpdate,
  siteUpdate
} from "$lib/server/deepwell/admin"
import { translate } from "$lib/server/deepwell/translate"
import { normalizeActionError } from "$lib/server/load/action-error"
import { adminView, type PreloadDataAsync } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { Layout } from "$lib/types"
import { error } from "@sveltejs/kit"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  boolean,
  literal,
  integer,
  maxValue,
  minValue,
  nullable,
  number,
  object,
  optional,
  pipe,
  string,
  maxLength,
  enum as vEnum
} from "valibot"

import type { TranslateKeys } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"

export async function loadAdminPage(
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync
) {
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")

  const parentData = await preloadData()
  const locales = parentData.locales

  const response = await adminView(siteId, locales, sessionToken)

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys,
    "footer-license-unless": {
      license: parentData.license_name,
      "license_url": parentData.license_url
    }
  }

  let errorStatus = null

  switch (response.type) {
    case "site_found":
      break
    case "admin_permissions":
      errorStatus = 401
      break
    default:
      // Unexpected response type!
      // There is an inconsistency between here / DEEPWELL
      errorStatus = 500
  }

  if (errorStatus === null) {
    translateKeys = {
      ...translateKeys,

      // Edit actions
      "edit": {},
      "save": {},
      "cancel": {},

      // Site info attributes
      "site-info.name": {},
      "site-info.slug": {},
      "site-info.tagline": {},
      "site-info.description": {},
      "site-info.default-page": {},
      "site-info.locale": {},
      "site-info.layout": {},
      "wiki-page-layout.default": {},
      "wiki-page-layout.wikidot": {},
      "wiki-page-layout.wikijump": {}
    }
  }

  const internationalization = await translate(locales, translateKeys)

  const adminForm = await superValidate(request, valibot(adminSchema))
  const navigationForm = await superValidate(request, valibot(navigationSchema))
  const licenseForm = await superValidate(request, valibot(licenseSchema))
  const templateForm = await superValidate(request, valibot(templateSchema))
  const ratingForm = await superValidate(request, valibot(ratingSchema))
  const siteIconsForm = await superValidate(request, valibot(siteIconsSchema))
  const forumNestingForm = await superValidate(request, valibot(forumNestingSchema))
  const discussionForm = await superValidate(request, valibot(discussionSchema))

  const viewData = {
    view: response.type,
    html: response.data?.html,
    internationalization,
    adminForm,
    navigationForm,
    licenseForm,
    templateForm,
    ratingForm,
    siteIconsForm,
    forumNestingForm,
    discussionForm,
    categories: response.type === "site_found" ? response.data.categories : [],
    pageTemplates: response.type === "site_found" ? response.data.page_templates : []
  }

  if (errorStatus !== null) {
    error(errorStatus, viewData)
  }

  return viewData
}

export async function siteIconsAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(siteIconsSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's icons"
    })
  }

  try {
    const res = await siteIconsUpdate(
      form.data.siteId,
      session.user_id,
      getClientAddress(),
      {
        faviconSource: emptyToNull(form.data.faviconSource),
        iosIconSource: emptyToNull(form.data.iosIconSource),
        windowsTileSource: emptyToNull(form.data.windowsTileSource)
      },
      { sessionToken, siteId: form.data.siteId }
    )
    return { form, res }
  } catch (error) {
    const details = error as {
      message?: string
      code?: string
      data?: Record<string, unknown>
    }
    return fail(500, {
      form,
      message: details.message,
      code: details.code,
      data: details.data
    })
  }
}

function emptyToNull(value: string): string | null {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

export async function forumNestingAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(forumNestingSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's forum settings"
    })
  }

  try {
    const res = await siteForumNestingUpdate(
      form.data.siteId,
      session.user_id,
      getClientAddress(),
      form.data.maxNestLevel,
      { sessionToken, siteId: form.data.siteId }
    )
    return { form, res }
  } catch (error) {
    const details = error as {
      message?: string
      code?: string
      data?: Record<string, unknown>
    }
    return fail(500, {
      form,
      message: details.message,
      code: details.code,
      data: details.data
    })
  }
}

export async function discussionAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(discussionSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's discussion settings"
    })
  }

  try {
    const res = await categoryDiscussionUpdate(
      form.data.siteId,
      form.data.categoryId,
      session.user_id,
      getClientAddress(),
      discussionUpdateValue(form.data),
      { sessionToken, siteId: form.data.siteId }
    )
    return { form, res }
  } catch (error) {
    const details = error as {
      message?: string
      code?: string
      data?: Record<string, unknown>
    }
    return fail(500, {
      form,
      message: details.message,
      code: details.code,
      data: details.data
    })
  }
}

export async function templateAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(templateSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's page templates"
    })
  }

  const { siteId, categoryId, templatePageId } = form.data
  try {
    const session = await authGetSession(sessionToken)
    const res = await categoryTemplateUpdate(
      {
        siteId,
        categoryId,
        userId: session.user_id,
        userIpAddr: getClientAddress(),
        templatePageId
      },
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    const details = normalizeActionError(error)
    return fail(500, {
      form,
      ...details
    })
  }
}

export async function licenseAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(licenseSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's license"
    })
  }

  const { siteId, categoryId } = form.data
  const { license, licenseOther } = licenseUpdateValue(form.data)
  try {
    const session = await authGetSession(sessionToken)
    const res = await categoryLicenseUpdate(
      {
        siteId,
        categoryId,
        userId: session.user_id,
        userIpAddr: getClientAddress(),
        license,
        licenseOther
      },
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    const details = normalizeActionError(error)
    return fail(500, {
      form,
      ...details
    })
  }
}

export async function ratingAction({ request, getClientAddress, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(ratingSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's rating settings"
    })
  }

  const { siteId, categoryId, inherit, enabled, permission, visibility, ratingType } =
    form.data
  try {
    const res = await categoryRatingUpdate(
      siteId,
      categoryId,
      session.user_id,
      getClientAddress(),
      inherit ? null : enabled,
      inherit ? null : permission,
      inherit ? null : visibility,
      inherit ? null : ratingType,
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    const details = error as {
      message?: string
      code?: string
      data?: Record<string, unknown>
    }
    return fail(500, {
      form,
      message: details.message,
      code: details.code,
      data: details.data
    })
  }
}

export async function navigationAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(navigationSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's navigation"
    })
  }

  const { siteId, categoryId } = form.data
  const { topBarPage, sideBarPage } = navigationUpdateValues(form.data)
  try {
    const session = await authGetSession(sessionToken)
    const res = await categoryNavigationUpdate(
      {
        siteId,
        categoryId,
        userId: session.user_id,
        userIpAddr: getClientAddress(),
        topBarPage,
        sideBarPage
      },
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    const details = normalizeActionError(error)
    return fail(500, {
      form,
      ...details
    })
  }
}

export async function adminAction({ request, getClientAddress, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(adminSchema))

  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  try {
    if (form.data.action === "edit") {
      if (!sessionToken) {
        return fail(401, {
          form,
          message: "user does not have permission to edit this site"
        })
      }
      const session = await authGetSession(sessionToken)

      const { name, slug, tagline, description, defaultPage, locale, layout, siteId } =
        form.data

      const res = await siteUpdate(
        {
          siteId,
          userId: session.user_id,
          userIpAddr: ipAddress,
          name,
          slug,
          tagline,
          description,
          defaultPage,
          locale,
          layout
        },
        { sessionToken, siteId }
      )

      return { form, res }
    }

    return { form, res: null }
  } catch (error) {
    const details = normalizeActionError(error)
    return fail(500, {
      form,
      ...details
    })
  }
}

const adminSchema = object({
  name: string(),
  slug: string(),
  tagline: string(),
  description: string(),
  defaultPage: string(),
  locale: string(),
  layout: vEnum(Layout),
  siteId: number(),
  action: optional(nullable(literal("edit")))
})

const navigationSchema = object({
  siteId: number(),
  categoryId: number(),
  inherit: boolean(),
  topBarPage: string(),
  sideBarPage: string()
})

const licenseSchema = object({
  siteId: number(),
  categoryId: number(),
  inherit: boolean(),
  license: string(),
  licenseOther: maxLength(string(), 300)
})

const templateSchema = object({
  siteId: number(),
  categoryId: number(),
  templatePageId: nullable(number())
})

const ratingSchema = object({
  siteId: number(),
  categoryId: number(),
  inherit: boolean(),
  enabled: boolean(),
  permission: vEnum({ REGISTERED: "registered", MEMBERS: "members" }),
  visibility: vEnum({ VISIBLE: "visible", ANONYMOUS: "anonymous" }),
  ratingType: vEnum({ PLUS: "plus", PLUS_MINUS: "plus_minus", STARS: "stars" })
})

// Wikidot accepts a local upload or an existing URL per icon slot. This slice
// records the source; an empty field clears the slot.
const siteIconsSchema = object({
  siteId: number(),
  faviconSource: optional(string(), ""),
  iosIconSource: optional(string(), ""),
  windowsTileSource: optional(string(), "")
})

const forumNestingSchema = object({
  siteId: number(),
  maxNestLevel: pipe(number(), integer(), minValue(0), maxValue(10))
})

const discussionSchema = object({
  siteId: number(),
  categoryId: number(),
  state: vEnum({ DEFAULT: "default", ENABLE: "enable", DISABLE: "disable" })
})
