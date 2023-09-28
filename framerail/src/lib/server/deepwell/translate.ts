import { wellfetch } from "$lib/server/deepwell/index.ts"

export interface PageRoute {
  slug: string
  extra: string
}

export async function translate(
  locale: string,
  keys: Record<string, Record<string, string|number>>|{}
): Promise<object> {
  const response = await wellfetch(`/message/${locale}/translate`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(keys)
  })

  if (!response.ok) {
    throw new Error("Unable to get translated strings from server")
  }

  return response.json()
}
