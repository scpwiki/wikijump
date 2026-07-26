import { fixtureState, hasExactKeys } from "./context.js"

/** @param {{ rpcRequest: any }} input */
export const handleSiteRpc = ({ rpcRequest }) => {
  let result

  if (
    rpcRequest.method === "category_get_all" &&
    rpcRequest.params?.site === "scp-wiki"
  ) {
    result = [{ slug: "_default" }, { slug: "nav" }]
  } else if (
    rpcRequest.method === "site_get" &&
    hasExactKeys(rpcRequest.params, ["site"]) &&
    (rpcRequest.params.site === "scp-wiki" || rpcRequest.params.site === "missing-site")
  ) {
    fixtureState.pageReadRequests.siteGet.push(rpcRequest.params)
    result = rpcRequest.params.site === "scp-wiki" ? { site_id: 6000005 } : null
  } else if (
    rpcRequest.method === "translate" &&
    hasExactKeys(rpcRequest.params, ["locales", "messages", "strip_message_keys"]) &&
    Array.isArray(rpcRequest.params.locales) &&
    typeof rpcRequest.params.messages === "object" &&
    rpcRequest.params.messages !== null &&
    Array.isArray(rpcRequest.params.strip_message_keys)
  ) {
    result = Object.fromEntries(
      Object.keys(rpcRequest.params.messages).map((key) => [key, key])
    )
  } else {
    return undefined
  }

  return { result }
}
