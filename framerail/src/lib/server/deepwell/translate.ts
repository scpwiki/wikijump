import { client } from "$lib/server/deepwell/index.ts"
import type { TranslateKeys } from "$lib/types"

export async function translate(locales: string[], keys: TranslateKeys): Promise<object> {
  return client.request("translate", {
    locales,
    messages: keys
  })
}
