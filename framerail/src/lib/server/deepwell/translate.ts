import defaults from "$lib/defaults"
import { client } from "$lib/server/deepwell"
import type { TranslateKeys, TranslatedKeys } from "$lib/types"

export async function translate(
  locales: string[],
  keys: TranslateKeys,
  stripKeys: string[] = defaults.translateStripKeys
): Promise<TranslatedKeys> {
  // Presentation batches resolve missing keys to null instead of failing
  // the whole request, so one missing key (for example from a stale locale
  // catalog during rollout) cannot turn a page into HTTP 500. Deepwell
  // still logs the missing key server-side for diagnosis.
  return client.request("translate", {
    locales,
    messages: keys,
    strip_message_keys: stripKeys,
    allow_missing: true
  })
}
