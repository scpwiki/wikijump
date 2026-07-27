import { page } from "$app/state"
import type { Locales } from "../../types"

export function format(messageKey: keyof Locales) {
  return (page.data?.internationalization ?? page.error?.internationalization)?.[
    messageKey
  ]
}
