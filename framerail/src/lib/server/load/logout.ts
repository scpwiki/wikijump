import defaults from "$lib/defaults"
import { translateWithFallback } from "$lib/server/deepwell/translate"
import { parse } from "accept-language-parser"

export async function loadLogoutPage(
  request,
  cookies
) {
  // Set up parameters
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  const language = request.headers.get("Accept-Language")
  let locales = parse(language).map((lang) =>
    lang.region ? `${lang.code}-${lang.region}` : lang.code
  )

  let viewData: Record<string, any> = {
    isLoggedIn: !!sessionToken
  }

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  let translateKeys: Record<string, Record<string, string | number> | {}> = {
    ...defaults.translateKeys,

    // Page actions
    "cancel": {},
    "logout": {},

    // misc
    "logout.toast": {}
  }

  const translated = await translateWithFallback(locales, translateKeys)

  viewData.internationalization = translated

  // Return to page for rendering
  return viewData
}