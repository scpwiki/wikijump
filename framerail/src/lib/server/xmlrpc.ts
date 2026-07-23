import { Buffer } from "node:buffer"
import { timingSafeEqual } from "node:crypto"

import { dispatchXmlRpcCall } from "$lib/server/xmlrpc-methods"
import {
  faultResponse,
  parseXmlRpcCall,
  serializeMethodResponse,
  xmlResponse,
  XmlRpcFault
} from "$lib/server/xmlrpc-protocol"

export type { XmlRpcCall, XmlRpcValue } from "$lib/server/xmlrpc-protocol"
export { parseXmlRpcCall, serializeMethodResponse } from "$lib/server/xmlrpc-protocol"

interface BasicAuthCredentials {
  username: string
  password: string
}

const MAX_XML_RPC_BODY_BYTES = 1_048_576
const XML_RPC_DEFAULT_REQUEST_IP = "127.0.0.1"

export async function handleXmlRpcRequest(
  request: Request,
  requestIp = XML_RPC_DEFAULT_REQUEST_IP
): Promise<Response> {
  if (request.method !== "POST") {
    return faultResponse(
      new XmlRpcFault(405, "XML-RPC endpoint requires POST", 405, {
        allow: "POST"
      })
    )
  }

  const auth = parseBasicAuth(request.headers.get("authorization"))
  if (!auth || !isAuthorizedBasicAuth(auth)) {
    return faultResponse(
      new XmlRpcFault(401, "Missing or invalid HTTP Basic authentication", 401, {
        "www-authenticate": 'Basic realm="Wikijump XML-RPC"'
      })
    )
  }

  try {
    const body = await readXmlRpcBody(request)
    const call = parseXmlRpcCall(body)
    const result = await dispatchXmlRpcCall(call, {
      allowMulticall: true,
      requestIp
    })
    return xmlResponse(serializeMethodResponse(result))
  } catch (error) {
    if (error instanceof XmlRpcFault) {
      return faultResponse(error)
    }

    console.error("Unexpected XML-RPC handler error", error)
    return faultResponse(new XmlRpcFault(-32700, "Malformed XML-RPC request"))
  }
}

async function readXmlRpcBody(request: Request): Promise<string> {
  const contentLength = request.headers.get("content-length")
  if (contentLength) {
    const normalizedContentLength = contentLength.trim()
    if (
      !/^\d+$/.test(normalizedContentLength) ||
      Number.parseInt(normalizedContentLength, 10) > MAX_XML_RPC_BODY_BYTES
    ) {
      throw new XmlRpcFault(413, "XML-RPC request body is too large", 413)
    }
  }

  if (!request.body) {
    return ""
  }

  const reader = request.body.getReader()
  const decoder = new TextDecoder("utf-8", { fatal: true })
  let bytesRead = 0
  let body = ""

  while (true) {
    const { done, value } = await reader.read()
    if (done) {
      break
    }

    bytesRead += value.byteLength
    if (bytesRead > MAX_XML_RPC_BODY_BYTES) {
      await reader.cancel()
      throw new XmlRpcFault(413, "XML-RPC request body is too large", 413)
    }

    try {
      body += decoder.decode(value, { stream: true })
    } catch {
      throw new XmlRpcFault(-32700, "Malformed XML-RPC request")
    }
  }

  try {
    return body + decoder.decode()
  } catch {
    throw new XmlRpcFault(-32700, "Malformed XML-RPC request")
  }
}

function isAuthorizedBasicAuth(credentials: BasicAuthCredentials): boolean {
  const xmlRpcUsername = process.env.XML_RPC_USERNAME
  const xmlRpcPassword = process.env.XML_RPC_PASSWORD
  if (Boolean(xmlRpcUsername) !== Boolean(xmlRpcPassword)) {
    return false
  }
  const useXmlRpcCredentials = Boolean(xmlRpcUsername && xmlRpcPassword)
  const username = useXmlRpcCredentials ? xmlRpcUsername : process.env.WIKIDOT_APP_NAME
  const password = useXmlRpcCredentials ? xmlRpcPassword : process.env.WIKIDOT_API_KEY
  if (!username || !password) {
    return false
  }

  return (
    timingSafeStringEquals(credentials.username, username) &&
    timingSafeStringEquals(credentials.password, password)
  )
}

function timingSafeStringEquals(actual: string, expected: string): boolean {
  const actualBuffer = Buffer.from(actual, "utf8")
  const expectedBuffer = Buffer.from(expected, "utf8")
  return (
    actualBuffer.length === expectedBuffer.length &&
    timingSafeEqual(actualBuffer, expectedBuffer)
  )
}

function parseBasicAuth(header: string | null): BasicAuthCredentials | null {
  const match = header?.match(/^basic\s+(.+)$/i)
  if (!match) {
    return null
  }

  try {
    const decoded = Buffer.from(match[1], "base64").toString("utf8")
    const separator = decoded.indexOf(":")
    if (separator <= 0 || separator === decoded.length - 1) {
      return null
    }

    return {
      username: decoded.slice(0, separator),
      password: decoded.slice(separator + 1)
    }
  } catch {
    return null
  }
}
