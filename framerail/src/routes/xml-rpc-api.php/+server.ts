import { handleXmlRpcRequest } from "$lib/server/xmlrpc"

import type { RequestHandler } from "./$types"

export const POST: RequestHandler = ({ request }) => handleXmlRpcRequest(request)
