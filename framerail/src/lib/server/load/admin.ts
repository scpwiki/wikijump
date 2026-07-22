import defaults from "$lib/defaults"
import { licenseUpdateValue } from "$lib/admin-license.js"
import { navigationUpdateValues } from "$lib/admin-navigation.js"

import { authGetSession } from "$lib/server/auth/getSession"
import {
  categoryLicenseUpdate,
  categoryNavigationUpdate,
  siteUpdate
} from "$lib/server/deepwell/admin"
import { translate } from "$lib/server/deepwell/translate"
import { adminView, type PreloadDataAsync } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { Layout } from "$lib/types"
import { error } from "@sveltejs/kit"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  boolean,
  literal,
  nullable,
  number,
  object,
  optional,
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

  const viewData = {
    view: response.type,
    html: response.data?.html,
    internationalization,
    adminForm,
    navigationForm,
    licenseForm,
    categories: response.type === "site_found" ? response.data.categories : []
  }

  if (errorStatus !== null) {
    error(errorStatus, viewData)
  }

  return viewData
}

export async function licenseAction({
  request,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(licenseSchema))
  if (!form.valid) return fail(400, { form })

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's license"
    })
  }

  const { siteId, categoryId } = form.data
  const { license, licenseOther } = licenseUpdateValue(form.data)
  try {
    const res = await categoryLicenseUpdate(
      siteId,
      categoryId,
      session.user_id,
      getClientAddress(),
      license,
      licenseOther,
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    return fail(500, {
      form,
      message: error?.message,
      code: error?.code,
      data: error?.data
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
  const session = await authGetSession(sessionToken)
  if (!sessionToken || !session) {
    return fail(401, {
      form,
      message: "user does not have permission to edit this site's navigation"
    })
  }

  const { siteId, categoryId } = form.data
  const { topBarPage, sideBarPage } = navigationUpdateValues(form.data)
  try {
    const res = await categoryNavigationUpdate(
      siteId,
      categoryId,
      session.user_id,
      getClientAddress(),
      topBarPage,
      sideBarPage,
      { sessionToken, siteId }
    )
    return { form, res }
  } catch (error) {
    return fail(500, {
      form,
      message: error?.message,
      code: error?.code,
      data: error?.data
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
  const session = await authGetSession(sessionToken)

  try {
    if (form.data.action === "edit") {
      if (!sessionToken || !session) {
        return fail(401, {
          form,
          message: "user does not have permission to edit this site"
        })
      }

      const { name, slug, tagline, description, defaultPage, locale, layout, siteId } =
        form.data

      const res = await siteUpdate(
        siteId,
        session?.user_id,
        ipAddress,
        name,
        slug,
        tagline,
        description,
        defaultPage,
        locale,
        layout,
        { sessionToken, siteId }
      )

      return { form, res }
    }

    return { form, res: null }
  } catch (error) {
    return fail(500, {
      form,
      message: error?.message,
      code: error?.code,
      data: error?.data
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
