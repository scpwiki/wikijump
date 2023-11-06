import defaults from "$lib/defaults"
import { translate } from "$lib/server/deepwell/translate"
import { userView } from "$lib/server/deepwell/user.ts"
import type { TranslateKeys } from "$lib/types"
import { error, redirect } from "@sveltejs/kit"
import { parse } from "accept-language-parser"

export async function loadUser(username?: string, request, cookies) {
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  const language = request.headers.get("Accept-Language")
  let locales = parse(language)
    .sort((a, b) => a.quality - b.quality)
    .map((lang) => (lang.region ? `${lang.code}-${lang.region}` : lang.code))

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  const response = await userView(domain, sessionToken, locales, username)

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys
  }

  const viewData = response.data
  viewData.view = response.type

  let errorStatus = null

  switch (response.type) {
    case "user_found":
      break
    case "user_missing":
      viewData.user = null
      errorStatus = 404
      break
    case "site_missing":
      errorStatus = 404
  }

  if (errorStatus === null && username && viewData.user.slug !== username) {
    throw redirect(308, `/-/user/${viewData.user.slug}`)
  }

  if (errorStatus !== null) {
    translateKeys = {
      ...translateKeys,
      "user-not-exist": {}
    }
  } else {
    translateKeys = {
      ...translateKeys,
      "user-profile-info.name": {},
      "user-profile-info.gender": {},
      "user-profile-info.birthday": {},
      "user-profile-info.location": {},
      "user-profile-info.biography": {},
      "user-profile-info.user-page": {}
    }
  }

  const translated = await translate(locales, translateKeys)

  viewData.internationalization = translated

  if (errorStatus !== null) {
    throw error(errorStatus, viewData)
  }

  return viewData
}
