import type { LanguageDescription } from "@codemirror/language"
import { languages } from "@codemirror/language-data"
import { Facet } from "@codemirror/state"

/**
 * A `Facet` that holds a list of `LanguageDescription` instances.
 * Languages can be added to this facet in the editor, so that a plugin may
 * retrieve a list of languages in common use by the editor and its plugins.
 */
export const languageList = Facet.define<LanguageDescription>()

/** Returns an extension for every `LanguageDescription` provided. */
export function addLanguages(...languages: LanguageDescription[]) {
  return languages.map(language => languageList.of(language))
}

/**
 * A list of extensions that adds every language from the
 * `@codemirror/language-data` package into the {@link languageList} facet.
 */
export const defaultLanguages = addLanguages(...languages)
