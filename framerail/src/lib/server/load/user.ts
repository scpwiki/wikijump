import defaults from "$lib/defaults"
import { translateWithFallback } from "$lib/server/deepwell/translate"
import { userView } from "$lib/server/deepwell/user.ts"
import { error, redirect } from "@sveltejs/kit"
import { parse } from "accept-language-parser"

export async function loadUser(
  username?: string,
  request,
  cookies
) {
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  const language = request.headers.get("Accept-Language")
  let locales = parse(language).map((lang) =>
    lang.region ? `${lang.code}-${lang.region}` : lang.code
  )

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  const response = await userView(domain, sessionToken, defaults.fallbackLocale, username)

  let translateKeys: Record<string, Record<string, string | number> | {}> = {
    ...defaults.translateKeys
  }

  const viewData = response.data
  viewData.view = response.type

  let errorStatus = null

  switch (response.type) {
    case "userFound":
      break
    case "userMissing":
      viewData.user = null
      errorStatus = 404
      break
    case "siteMissing":
      errorStatus = 404
  }
  
  if (errorStatus === null && username && viewData.user.slug !== username) {
    throw redirect(308, `/-/user/${viewData.user.slug}`)
  }

  if (errorStatus !== null) {
    translateKeys = Object.assign(translateKeys, {
      "user-not-exist": {}
    })
  }

  const translated = await translateWithFallback(locales, translateKeys)

  viewData.internationalization = translated

  if (errorStatus !== null) {
    throw error(errorStatus, viewData)
  }

  return viewData
}