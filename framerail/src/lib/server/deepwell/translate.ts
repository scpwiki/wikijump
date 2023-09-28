import { client } from "$lib/server/deepwell/index.ts"

export interface PageRoute {
  slug: string
  extra: string
}

export async function translate(
  locale: string,
  keys: Record<string, Record<string, string | number>> | {}
): Promise<object> {
  return client.request("translate", {
    locale,
    messages: keys
  })
}
