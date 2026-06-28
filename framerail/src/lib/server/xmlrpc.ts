import { Buffer } from "node:buffer"
import { timingSafeEqual } from "node:crypto"

type XmlRpcScalar = string | number | boolean | null
export type XmlRpcValue = XmlRpcScalar | XmlRpcValue[] | Record<string, XmlRpcValue>

export interface XmlRpcCall {
  methodName: string
  params: XmlRpcValue[]
}

interface XmlElement {
  content: string
  end: number
  selfClosing: boolean
  start: number
}

interface BasicAuthCredentials {
  username: string
  password: string
}

const XML_RPC_HEADERS = {
  "content-type": "text/xml; charset=utf-8"
}

const XML_RPC_INT_MIN = -2_147_483_648
const XML_RPC_INT_MAX = 2_147_483_647
const MAX_XML_RPC_BODY_BYTES = 1_048_576
const XML_WHITESPACE = "[ \\t\\r\\n]"
const SYSTEM_METHODS = ["system.listMethods"]

class XmlRpcFault extends Error {
  constructor(
    readonly faultCode: number,
    readonly faultString: string,
    readonly httpStatus = 200,
    readonly headers: Record<string, string> = {}
  ) {
    super(faultString)
  }
}

export async function handleXmlRpcRequest(request: Request): Promise<Response> {
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
    const result = await dispatchXmlRpcCall(call)
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

async function dispatchXmlRpcCall(call: XmlRpcCall): Promise<XmlRpcValue> {
  switch (call.methodName) {
    case "system.listMethods":
      if (call.params.length !== 0) {
        throw new XmlRpcFault(-32602, "system.listMethods does not accept parameters")
      }
      return SYSTEM_METHODS
    default:
      throw new XmlRpcFault(-32601, `Unsupported XML-RPC method: ${call.methodName}`)
  }
}

function isAuthorizedBasicAuth(credentials: BasicAuthCredentials): boolean {
  const username = process.env.XML_RPC_USERNAME
  const password = process.env.XML_RPC_PASSWORD
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

export function parseXmlRpcCall(xml: string): XmlRpcCall {
  const normalized = stripIgnorableXml(xml)
  const methodCall = extractFirstDirectElement(normalized, "methodCall")
  if (!methodCall) {
    throw new XmlRpcFault(-32600, "Missing XML-RPC <methodCall> element")
  }
  const methodNameElement = extractRequiredElement(methodCall.content, "methodName")
  rejectSkippedXmlContent(methodCall.content, 0, methodNameElement.start, "methodCall")
  const methodName = trimXmlWhitespace(decodeXmlText(methodNameElement.content))
  if (methodName.length === 0) {
    throw new XmlRpcFault(-32600, "XML-RPC methodName must not be empty")
  }

  const paramsElement = extractOptionalElement(
    methodCall.content,
    "params",
    methodNameElement.end
  )
  if (paramsElement) {
    rejectSkippedXmlContent(
      methodCall.content,
      methodNameElement.end,
      paramsElement.start,
      "methodCall"
    )
    rejectSkippedXmlContent(
      methodCall.content,
      paramsElement.end,
      methodCall.content.length,
      "methodCall"
    )
  } else {
    rejectSkippedXmlContent(
      methodCall.content,
      methodNameElement.end,
      methodCall.content.length,
      "methodCall"
    )
  }

  if (!paramsElement || paramsElement.selfClosing) {
    return { methodName, params: [] }
  }

  const params: XmlRpcValue[] = []
  let offset = 0
  while (true) {
    const param = extractOptionalElement(paramsElement.content, "param", offset)
    if (!param) {
      break
    }

    rejectSkippedXmlContent(paramsElement.content, offset, param.start, "params")
    const value = extractRequiredElement(param.content, "value")
    rejectSkippedXmlContent(param.content, 0, value.start, "param")
    rejectSkippedXmlContent(param.content, value.end, param.content.length, "param")
    params.push(parseXmlRpcValue(value.content))
    offset = param.end
  }
  rejectSkippedXmlContent(
    paramsElement.content,
    offset,
    paramsElement.content.length,
    "params"
  )

  return { methodName, params }
}

function parseXmlRpcValue(valueContent: string): XmlRpcValue {
  const text = trimXmlWhitespace(valueContent)
  if (!text.startsWith("<")) {
    return decodeXmlText(valueContent)
  }

  if (isSelfClosingElement(text, "nil")) {
    return null
  }

  const stringElement = extractFirstDirectElement(text, "string")
  if (stringElement) {
    return decodeXmlText(stringElement.content)
  }

  const intElement =
    extractFirstDirectElement(text, "int") ?? extractFirstDirectElement(text, "i4")
  if (intElement) {
    const token = trimXmlWhitespace(decodeXmlText(intElement.content))
    if (!/^[+-]?\d+$/.test(token)) {
      throw new XmlRpcFault(-32602, "Invalid XML-RPC integer value")
    }
    const value = Number.parseInt(token, 10)
    if (value < XML_RPC_INT_MIN || value > XML_RPC_INT_MAX) {
      throw new XmlRpcFault(-32602, "Invalid XML-RPC integer value")
    }
    return value
  }

  const booleanElement = extractFirstDirectElement(text, "boolean")
  if (booleanElement) {
    const value = trimXmlWhitespace(decodeXmlText(booleanElement.content))
    if (value === "1") return true
    if (value === "0") return false
    throw new XmlRpcFault(-32602, "Invalid XML-RPC boolean value")
  }

  const doubleElement = extractFirstDirectElement(text, "double")
  if (doubleElement) {
    const token = trimXmlWhitespace(decodeXmlText(doubleElement.content))
    if (!/^[+-]?(?:\d+\.\d*|\.\d+)$/.test(token)) {
      throw new XmlRpcFault(-32602, "Invalid XML-RPC double value")
    }
    const value = Number.parseFloat(token)
    if (!Number.isFinite(value)) {
      throw new XmlRpcFault(-32602, "Invalid XML-RPC double value")
    }
    return value
  }

  const base64Element = extractFirstDirectElement(text, "base64")
  if (base64Element) {
    throw new XmlRpcFault(-32602, "Unsupported XML-RPC value type")
  }

  const dateElement = extractFirstDirectElement(text, "dateTime.iso8601")
  if (dateElement) {
    throw new XmlRpcFault(-32602, "Unsupported XML-RPC value type")
  }

  const arrayElement = extractFirstDirectElement(text, "array")
  if (arrayElement) {
    const dataElement = extractRequiredElement(arrayElement.content, "data")
    rejectSkippedXmlContent(arrayElement.content, 0, dataElement.start, "array")
    rejectSkippedXmlContent(
      arrayElement.content,
      dataElement.end,
      arrayElement.content.length,
      "array"
    )
    const values: XmlRpcValue[] = []
    let offset = 0
    while (true) {
      const item = extractOptionalElement(dataElement.content, "value", offset)
      if (!item) {
        break
      }
      rejectSkippedXmlContent(dataElement.content, offset, item.start, "data")
      values.push(parseXmlRpcValue(item.content))
      offset = item.end
    }
    rejectSkippedXmlContent(
      dataElement.content,
      offset,
      dataElement.content.length,
      "data"
    )
    return values
  }

  const structElement = extractFirstDirectElement(text, "struct")
  if (structElement) {
    const values: Record<string, XmlRpcValue> = {}
    let offset = 0
    while (true) {
      const member = extractOptionalElement(structElement.content, "member", offset)
      if (!member) {
        break
      }

      rejectSkippedXmlContent(structElement.content, offset, member.start, "struct")
      const nameElement = extractRequiredElement(member.content, "name")
      rejectSkippedXmlContent(member.content, 0, nameElement.start, "member")
      const name = decodeXmlText(nameElement.content)
      const value = extractRequiredElement(member.content, "value", nameElement.end)
      rejectSkippedXmlContent(member.content, nameElement.end, value.start, "member")
      rejectSkippedXmlContent(member.content, value.end, member.content.length, "member")
      values[name] = parseXmlRpcValue(value.content)
      offset = member.end
    }
    rejectSkippedXmlContent(
      structElement.content,
      offset,
      structElement.content.length,
      "struct"
    )
    return values
  }

  throw new XmlRpcFault(-32602, "Unsupported XML-RPC value type")
}

function extractFirstDirectElement(text: string, tagName: string): XmlElement | null {
  const trimmed = trimXmlWhitespace(text)
  const element = extractOptionalElement(trimmed, tagName)
  if (!element) {
    return null
  }

  const prefix = trimXmlWhitespace(trimmed.slice(0, element.start))
  const suffix = trimXmlWhitespace(trimmed.slice(element.end))
  return prefix.length === 0 && suffix.length === 0 ? element : null
}

function extractRequiredElement(text: string, tagName: string, offset = 0): XmlElement {
  const element = extractOptionalElement(text, tagName, offset)
  if (!element) {
    throw new XmlRpcFault(-32600, `Missing XML-RPC <${tagName}> element`)
  }
  return element
}

function extractOptionalElement(
  text: string,
  tagName: string,
  offset = 0
): XmlElement | null {
  const openPattern = new RegExp(
    `<${escapeRegExp(tagName)}(?:${XML_WHITESPACE}[^>]*)?>|<${escapeRegExp(
      tagName
    )}${XML_WHITESPACE}*/>`,
    "g"
  )
  openPattern.lastIndex = offset
  const match = openPattern.exec(text)
  if (!match) {
    return null
  }

  const opening = match[0]
  const contentStart = match.index + opening.length
  if (opening.endsWith("/>")) {
    return { content: "", end: contentStart, selfClosing: true, start: match.index }
  }

  const tagPattern = new RegExp(
    `</?${escapeRegExp(tagName)}(?:${XML_WHITESPACE}[^>]*)?>|<${escapeRegExp(
      tagName
    )}${XML_WHITESPACE}*/>`,
    "g"
  )
  tagPattern.lastIndex = contentStart
  let depth = 1

  while (true) {
    const tagMatch = tagPattern.exec(text)
    if (!tagMatch) {
      throw new XmlRpcFault(-32600, `Unclosed XML-RPC <${tagName}> element`)
    }

    const tag = tagMatch[0]
    if (tag.startsWith(`</${tagName}`)) {
      depth -= 1
      if (depth === 0) {
        return {
          content: text.slice(contentStart, tagMatch.index),
          end: tagPattern.lastIndex,
          selfClosing: false,
          start: match.index
        }
      }
    } else if (!tag.endsWith("/>")) {
      depth += 1
    }
  }
}

function rejectSkippedXmlContent(
  text: string,
  start: number,
  end: number,
  containerName: string
): void {
  if (!isXmlWhitespaceOnly(text.slice(start, end))) {
    throw new XmlRpcFault(
      -32600,
      `Unexpected XML-RPC content in <${containerName}> element`
    )
  }
}

function isSelfClosingElement(text: string, tagName: string): boolean {
  return new RegExp(`^<${escapeRegExp(tagName)}${XML_WHITESPACE}*/>$`).test(text)
}

function stripIgnorableXml(xml: string): string {
  if (xml.includes("<!--")) {
    throw new XmlRpcFault(-32600, "XML-RPC comments are not supported")
  }

  return trimXmlWhitespace(xml.replace(/^\uFEFF/, "").replace(/^<\?xml[\s\S]*?\?>/i, ""))
}

export function serializeMethodResponse(value: XmlRpcValue): string {
  return xmlDocument(
    `<methodResponse><params><param>${serializeValue(value)}</param></params></methodResponse>`
  )
}

function serializeFault(fault: XmlRpcFault): string {
  return xmlDocument(
    `<methodResponse><fault>${serializeValue({
      faultCode: fault.faultCode,
      faultString: fault.faultString
    })}</fault></methodResponse>`
  )
}

function serializeValue(value: XmlRpcValue): string {
  if (value === null) {
    return "<value><nil /></value>"
  }

  if (Array.isArray(value)) {
    return `<value><array><data>${value.map(serializeValue).join("")}</data></array></value>`
  }

  switch (typeof value) {
    case "string":
      return `<value><string>${escapeXmlText(value)}</string></value>`
    case "number":
      if (
        Number.isInteger(value) &&
        value >= XML_RPC_INT_MIN &&
        value <= XML_RPC_INT_MAX
      ) {
        return `<value><int>${value}</int></value>`
      }
      return `<value><double>${formatXmlRpcDouble(value)}</double></value>`
    case "boolean":
      return `<value><boolean>${value ? "1" : "0"}</boolean></value>`
    case "object":
      return `<value><struct>${Object.entries(value)
        .map(
          ([key, memberValue]) =>
            `<member><name>${escapeXmlText(key)}</name>${serializeValue(memberValue)}</member>`
        )
        .join("")}</struct></value>`
  }
}

function faultResponse(fault: XmlRpcFault): Response {
  return xmlResponse(serializeFault(fault), fault.httpStatus, fault.headers)
}

function xmlResponse(
  body: string,
  status = 200,
  headers: Record<string, string> = {}
): Response {
  return new Response(body, {
    status,
    headers: {
      ...XML_RPC_HEADERS,
      ...headers
    }
  })
}

function xmlDocument(body: string): string {
  return `<?xml version="1.0"?>${body}`
}

function escapeXmlText(value: string): string {
  assertValidXmlText(value, -32603, "Cannot serialize XML-RPC string value")
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
}

function formatXmlRpcDouble(value: number): string {
  if (!Number.isFinite(value) || Math.abs(value) >= 1e21) {
    throw new XmlRpcFault(-32603, "Cannot serialize XML-RPC double value")
  }

  const decimal = value.toString().includes("e")
    ? expandExponentialNumber(value)
    : value.toString()
  return decimal.includes(".") ? decimal : `${decimal}.0`
}

function expandExponentialNumber(value: number): string {
  const [mantissa, exponentText] = value.toString().toLowerCase().split("e")
  const exponent = Number.parseInt(exponentText, 10)
  const sign = mantissa.startsWith("-") ? "-" : ""
  const unsignedMantissa = sign ? mantissa.slice(1) : mantissa
  const [whole, fraction = ""] = unsignedMantissa.split(".")
  const digits = `${whole}${fraction}`
  const decimalIndex = whole.length + exponent

  if (decimalIndex <= 0) {
    return `${sign}0.${"0".repeat(Math.abs(decimalIndex))}${digits}`
  }

  if (decimalIndex >= digits.length) {
    return `${sign}${digits}${"0".repeat(decimalIndex - digits.length)}`
  }

  return `${sign}${digits.slice(0, decimalIndex)}.${digits.slice(decimalIndex)}`
}

function decodeXmlText(value: string): string {
  if (
    value.includes("<") ||
    /&(?!lt;|gt;|quot;|apos;|amp;|#x[0-9a-fA-F]+;|#\d+;)/.test(value)
  ) {
    throw new XmlRpcFault(-32600, "Invalid XML character data")
  }
  assertValidXmlText(value, -32600, "Invalid XML character data")

  const decoded = value.replace(
    /&(lt|gt|quot|apos|amp|#x[0-9a-fA-F]+|#\d+);/g,
    (entity) => {
      switch (entity) {
        case "&lt;":
          return "<"
        case "&gt;":
          return ">"
        case "&quot;":
          return '"'
        case "&apos;":
          return "'"
        case "&amp;":
          return "&"
        default: {
          const codePoint = entity.startsWith("&#x")
            ? Number.parseInt(entity.slice(3, -1), 16)
            : Number.parseInt(entity.slice(2, -1), 10)
          if (!Number.isInteger(codePoint)) {
            throw new XmlRpcFault(-32600, "Invalid XML character reference")
          }
          if (!isValidXmlCodePoint(codePoint)) {
            throw new XmlRpcFault(-32600, "Invalid XML character reference")
          }

          try {
            return String.fromCodePoint(codePoint)
          } catch {
            throw new XmlRpcFault(-32600, "Invalid XML character reference")
          }
        }
      }
    }
  )
  assertValidXmlText(decoded, -32600, "Invalid XML character data")
  return decoded
}

function trimXmlWhitespace(value: string): string {
  return value.replace(/^[ \t\r\n]+|[ \t\r\n]+$/g, "")
}

function isXmlWhitespaceOnly(value: string): boolean {
  return /^[ \t\r\n]*$/.test(value)
}

function assertValidXmlText(value: string, faultCode: number, faultString: string): void {
  for (const character of value) {
    if (!isValidXmlCodePoint(character.codePointAt(0) ?? 0)) {
      throw new XmlRpcFault(faultCode, faultString)
    }
  }
}

function isValidXmlCodePoint(codePoint: number): boolean {
  return (
    codePoint === 0x09 ||
    codePoint === 0x0a ||
    codePoint === 0x0d ||
    (codePoint >= 0x20 && codePoint <= 0xd7ff) ||
    (codePoint >= 0xe000 && codePoint <= 0xfffd) ||
    (codePoint >= 0x10000 && codePoint <= 0x10ffff)
  )
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}
