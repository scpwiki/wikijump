import { isMissingLocaleMessageError } from "../deepwell/public-error.js"

import type { TranslateKeys, TranslatedKeys } from "$lib/types"

type Translator = (locales: string[], keys: TranslateKeys) => Promise<TranslatedKeys>

export async function translateMfaCodeLabel(
  locales: string[],
  translator: Translator
): Promise<string> {
  try {
    const translated = await translator(locales, { "mfa-code": {} })
    const label = translated["mfa-code"]
    return typeof label === "string" && label.trim() ? label : "MFA code"
  } catch (error) {
    if (isMissingLocaleMessageError(error)) return "MFA code"
    throw error
  }
}
