import { parse } from "accept-language-parser"

export function parseAcceptLangHeader(req) {
  const language = req.headers.get("Accept-Language")
  let locales = parse(language)
    .sort((a, b) => b.quality - a.quality)
    .map((lang) => {
      let parts = [lang.code]
      if (lang.script) parts.push(lang.script)
      if (lang.region) parts.push(lang.region)
      return parts.join("-")
    })
  return locales
}
