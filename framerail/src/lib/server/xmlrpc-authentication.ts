import { client } from "$lib/server/deepwell"
import { xmlRpcAuthenticationConfiguration } from "$lib/server/xmlrpc-configuration"
import {
  requestXmlRpcDeepwell,
  type DeepwellRequestContext
} from "$lib/server/xmlrpc-deepwell-client"
import { XmlRpcFault, type XmlRpcValue } from "$lib/server/xmlrpc-protocol"

interface DeepwellLoginOutput {
  session_token: string
}

interface DeepwellSession {
  user_id: number
}

interface DeepwellUser {
  user_id: number
  name: string
  slug: string
}

export interface XmlRpcWriteContext extends DeepwellRequestContext {
  sessionToken: string
  siteId: number
  page: string
  userId: number
  ipAddress: string
}

export async function getXmlRpcWriteContext(
  siteId: number,
  page: string,
  requestIp: string
): Promise<XmlRpcWriteContext> {
  const principal = await getXmlRpcWritePrincipal(requestIp)

  return {
    sessionToken: principal.sessionToken,
    siteId,
    page,
    userId: principal.userId,
    ipAddress: requestIp
  }
}

export async function getAuthenticatedUser(
  requestIp: string
): Promise<Record<string, XmlRpcValue>> {
  const { ownerUsername } = xmlRpcAuthenticationConfiguration()
  const principal = ownerUsername
    ? {
        context: undefined,
        user: ownerUsername
      }
    : await getXmlRpcWriteUserLookup(requestIp)
  const user = await requestXmlRpcDeepwell(
    "user_get",
    { user: principal.user },
    principal.context
  )
  if (!isDeepwellUser(user)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: user_get")
  }

  return {
    name: user.slug,
    title: user.name,
    id: user.user_id
  }
}

async function getXmlRpcWriteUserLookup(
  requestIp: string
): Promise<{ context: DeepwellRequestContext; user: number }> {
  const principal = await getXmlRpcWritePrincipal(requestIp)

  return {
    context: { sessionToken: principal.sessionToken },
    user: principal.userId
  }
}

export async function getXmlRpcWritePrincipal(
  requestIp: string
): Promise<{ sessionToken: string; userId: number }> {
  const { writePassword, writeUsername } = xmlRpcAuthenticationConfiguration()
  if (!writeUsername || !writePassword) {
    throw new XmlRpcFault(
      403,
      "XML-RPC write actor authentication is not configured",
      403
    )
  }

  let login: DeepwellLoginOutput
  try {
    login = (await client.request("login", {
      name_or_email: writeUsername,
      password: writePassword,
      ip_address: requestIp,
      user_agent: "wikijump-xmlrpc-api/0.1"
    })) as DeepwellLoginOutput
  } catch {
    throw new XmlRpcFault(403, "Invalid XML-RPC write actor authentication", 403)
  }

  if (!isDeepwellLoginOutput(login)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: login")
  }

  const session = await requestXmlRpcDeepwell("session_get", [login.session_token])
  if (!isDeepwellSession(session)) {
    throw new XmlRpcFault(-32603, "XML-RPC login did not return a usable session")
  }

  return {
    sessionToken: login.session_token,
    userId: session.user_id
  }
}

function isDeepwellLoginOutput(value: unknown): value is DeepwellLoginOutput {
  return (
    isXmlRpcStruct(value) &&
    typeof value.session_token === "string" &&
    value.session_token.length > 0
  )
}

function isDeepwellSession(value: unknown): value is DeepwellSession {
  return (
    isXmlRpcStruct(value) &&
    typeof value.user_id === "number" &&
    Number.isInteger(value.user_id)
  )
}

function isDeepwellUser(value: unknown): value is DeepwellUser {
  return (
    isXmlRpcStruct(value) &&
    typeof value.user_id === "number" &&
    Number.isInteger(value.user_id) &&
    typeof value.name === "string" &&
    typeof value.slug === "string"
  )
}

function isXmlRpcStruct(value: unknown): value is Record<string, XmlRpcValue> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
