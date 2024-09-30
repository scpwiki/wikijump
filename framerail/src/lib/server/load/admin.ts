import defaults from "$lib/defaults"
import { parseAcceptLangHeader } from "$lib/locales"
import { translate } from "$lib/server/deepwell/translate"
import { adminView } from "$lib/server/deepwell/views"
import type { TranslateKeys } from "$lib/types"
import { error } from "@sveltejs/kit"

export async function loadAdminPage(request, cookies) {
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  let locales = parseAcceptLangHeader(request)

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  const response = await adminView(domain, locales, sessionToken)

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys
  }

  const viewData = response.data
  viewData.view = response.type

  let errorStatus = null

  switch (response.type) {
    case "site_found":
      break
    case "admin_permissions":
      errorStatus = 401
      break
    case "site_missing":
      errorStatus = 404
  }

  if (errorStatus !== null) {
    translateKeys = {
      ...translateKeys,
      "site-not-exist": {}
    }
  } else {
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
      "site-info.locale": {},
      "site-info.layout": {},
      "wiki-page-layout.default": {},
      "wiki-page-layout.wikidot": {},
      "wiki-page-layout.wikijump": {}
    }
  }

  const translated = await translate(locales, translateKeys)

  viewData.internationalization = translated

  if (errorStatus !== null) {
    throw error(errorStatus, viewData)
  }

  return viewData
}
