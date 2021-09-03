import type { LRParser, ParserConfig } from "@lezer/lr"
import {
  Extension,
  LanguageDescription,
  LanguageSupport,
  LRLanguage
} from "wj-codemirror/cm"

export interface CreateLezerLanguageOpts {
  name: string
  parser: LRParser
  configure?: ParserConfig
  alias?: string[]
  ext?: string[]
  languageData?: Record<string, any>
  extensions?: Extension[]
}

export function createLezerLanguage(opts: CreateLezerLanguageOpts) {
  const langDesc = Object.assign(
    { name: opts.name },
    opts.alias ? { alias: opts.alias } : {},
    opts.ext ? { extensions: opts.ext } : {}
  )
  const langData = { ...langDesc, ...(opts.languageData ?? {}) }

  const load = function () {
    const lang = LRLanguage.define({
      parser: opts.parser.configure(opts.configure ?? {}),
      languageData: langData
    })
    return new LanguageSupport(lang, opts.extensions)
  }

  const description = LanguageDescription.of({ ...langDesc, load: async () => load() })

  return { load, description }
}
