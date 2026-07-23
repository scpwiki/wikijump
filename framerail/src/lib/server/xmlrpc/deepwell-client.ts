import { client } from "$lib/server/deepwell"
import { XmlRpcFault } from "$lib/server/xmlrpc/protocol"

export interface DeepwellRequestContext {
  sessionToken?: string
  siteId?: number
  page?: string
}

export async function requestXmlRpcDeepwell(
  method: string,
  params: unknown,
  context?: DeepwellRequestContext
): Promise<unknown> {
  try {
    return await client.request(method, params, context)
  } catch (error) {
    console.error(`Unexpected Deepwell XML-RPC bridge error for ${method}`, error)
    throw new XmlRpcFault(-32603, `XML-RPC Deepwell request failed: ${method}`)
  }
}
