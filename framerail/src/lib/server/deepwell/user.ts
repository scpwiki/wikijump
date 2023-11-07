import { client } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function userView(
  domain: string,
  sessionToken: Optional<string>,
  locales: string[],
  username?: string
): Promise<object> {
  return client.request("user_view", {
    domain,
    session_token: sessionToken,
    locales,
    user: username
  })
}
