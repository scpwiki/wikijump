import { Buffer } from "node:buffer"
import { timingSafeEqual } from "node:crypto"

import { client } from "$lib/server/deepwell"

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

interface MethodDefinition {
  help: string
  signatures: string[][]
}

interface DeepwellCategory {
  slug: string
}

type DeepwellStringParams = {
  [key: string]: string | string[] | undefined
}

const XML_RPC_HEADERS = {
  "content-type": "text/xml; charset=utf-8"
}

const XML_RPC_INT_MIN = -2_147_483_648
const XML_RPC_INT_MAX = 2_147_483_647
const MAX_XML_RPC_BODY_BYTES = 1_048_576
const MAX_XML_RPC_MULTICALLS = 100
const MAX_XML_RPC_FILTER_VALUES = 100
const PAGE_SELECT_DECIMAL_RATING_PATTERN =
  /^[+-]?(?:(?:\d+(?:\.\d*)?)|(?:\.\d+))(?:[eE][+-]?\d+)?$/
const XML_WHITESPACE = "[ \\t\\r\\n]"
const METHOD_DEFINITIONS: Record<string, MethodDefinition> = {
  "system.listMethods": {
    help: "List XML-RPC methods exposed by this Wikijump endpoint.",
    signatures: [["array"]]
  },
  "system.methodHelp": {
    help: "Return help text for an XML-RPC method.",
    signatures: [["string", "string"]]
  },
  "system.methodSignature": {
    help: "Return XML-RPC signature metadata for a method.",
    signatures: [["array", "string"]]
  },
  "system.multicall": {
    help: "Execute multiple XML-RPC calls and return per-call results or faults.",
    signatures: [["array", "array"]]
  },
  "categories.select": {
    help: "Select categories from a Wikidot-compatible site.",
    signatures: [["array", "struct"]]
  },
  "tags.select": {
    help: "Select tags from a Wikidot-compatible site.",
    signatures: [["array", "struct"]]
  },
  "pages.select": {
    help: "Select pages from a Wikidot-compatible site.",
    signatures: [["array", "struct"]]
  },
  "pages.get_meta": {
    help: "Fetch metadata for a batch of Wikidot-compatible pages.",
    signatures: [["struct", "struct"]]
  },
  "pages.get_one": {
    help: "Fetch one Wikidot-compatible page.",
    signatures: [["struct", "struct"]]
  },
  "pages.save_one": {
    help: "Create or update one Wikidot-compatible page.",
    signatures: [["struct", "struct"]]
  },
  "files.select": {
    help: "Select files attached to a Wikidot-compatible page.",
    signatures: [["array", "struct"]]
  },
  "files.get_meta": {
    help: "Fetch metadata for Wikidot-compatible page files.",
    signatures: [["struct", "struct"]]
  },
  "files.get_one": {
    help: "Fetch one Wikidot-compatible page file.",
    signatures: [["struct", "struct"]]
  },
  "files.save_one": {
    help: "Create or update one Wikidot-compatible page file.",
    signatures: [["struct", "struct"]]
  },
  "users.get_me": {
    help: "Return the authenticated Wikidot-compatible API user.",
    signatures: [["struct"]]
  },
  "posts.select": {
    help: "Select Wikidot-compatible forum posts.",
    signatures: [["array", "struct"]]
  },
  "posts.get": {
    help: "Fetch one Wikidot-compatible forum post.",
    signatures: [["struct", "struct"]]
  }
}
const METHOD_NAMES = Object.keys(METHOD_DEFINITIONS)
const hasMethodDefinition = (methodName: string): boolean =>
  Object.prototype.hasOwnProperty.call(METHOD_DEFINITIONS, methodName)

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

async function dispatchXmlRpcCall(
  call: XmlRpcCall,
  options = { allowMulticall: true }
): Promise<XmlRpcValue> {
  switch (call.methodName) {
    case "system.listMethods":
      expectParamCount(call, 0)
      return METHOD_NAMES
    case "system.methodHelp":
      expectParamCount(call, 1)
      return getMethodDefinition(getStringParam(call, 0, "methodName")).help
    case "system.methodSignature":
      expectParamCount(call, 1)
      return getMethodDefinition(getStringParam(call, 0, "methodName")).signatures
    case "system.multicall":
      if (!options.allowMulticall) {
        throw new XmlRpcFault(-32600, "Nested system.multicall calls are not supported")
      }
      expectParamCount(call, 1)
      return dispatchMulticall(call)
    case "categories.select":
      expectParamCount(call, 1)
      return selectCategories(call)
    case "tags.select":
      expectParamCount(call, 1)
      return selectTags(call)
    case "pages.select":
      expectParamCount(call, 1)
      return selectPages(call)
    default:
      if (hasMethodDefinition(call.methodName)) {
        throw new XmlRpcFault(
          -32601,
          `XML-RPC method is not implemented yet: ${call.methodName}`
        )
      }
      throw new XmlRpcFault(-32601, `Unsupported XML-RPC method: ${call.methodName}`)
  }
}

async function dispatchMulticall(call: XmlRpcCall): Promise<XmlRpcValue[]> {
  const calls = getArrayParam(call, 0, "calls")
  if (calls.length > MAX_XML_RPC_MULTICALLS) {
    throw new XmlRpcFault(
      -32602,
      `system.multicall accepts at most ${MAX_XML_RPC_MULTICALLS} calls`
    )
  }

  const results: XmlRpcValue[] = []

  for (const child of calls) {
    try {
      if (!isXmlRpcStruct(child)) {
        throw new XmlRpcFault(-32602, "Each system.multicall entry must be a struct")
      }

      const methodName = child.methodName
      const params = child.params ?? []
      if (typeof methodName !== "string") {
        throw new XmlRpcFault(-32602, "Each system.multicall entry needs a methodName")
      }
      if (!Array.isArray(params)) {
        throw new XmlRpcFault(-32602, "system.multicall params must be an array")
      }

      const value = await dispatchXmlRpcCall(
        { methodName, params },
        { allowMulticall: false }
      )
      results.push([value])
    } catch (error) {
      let fault: XmlRpcFault
      if (error instanceof XmlRpcFault) {
        fault = error
      } else {
        console.error("Unexpected system.multicall child error", error)
        fault = new XmlRpcFault(-32603, "system.multicall child call failed")
      }
      results.push({
        faultCode: fault.faultCode,
        faultString: fault.faultString
      })
    }
  }

  return results
}

async function selectCategories(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const categories = expectDeepwellCategories(
    await requestDeepwell("category_get_all", {
      site
    }),
    "category_get_all"
  )

  return categories.map((category) => category.slug)
}

function expectDeepwellCategories(value: unknown, method: string): DeepwellCategory[] {
  if (
    !Array.isArray(value) ||
    value.some(
      (category) =>
        !isXmlRpcStruct(category) ||
        typeof category.slug !== "string" ||
        category.slug.length === 0
    )
  ) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value as DeepwellCategory[]
}

function expectDeepwellStringArray(value: unknown, method: string): string[] {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value
}

async function selectTags(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const categories = getOptionalStructStringArray(params, "categories")
  const pages = getOptionalStructStringArray(params, "pages")

  if (categories && categories.length > MAX_XML_RPC_FILTER_VALUES) {
    throw new XmlRpcFault(
      -32602,
      `tags.select categories is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
    )
  }
  if (pages && pages.length > MAX_XML_RPC_FILTER_VALUES) {
    throw new XmlRpcFault(
      -32602,
      `tags.select pages is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
    )
  }

  const deepwellParams: {
    site: string
    categories?: string[]
    pages?: string[]
  } = { site }
  if (categories) {
    deepwellParams.categories = categories
  }
  if (pages) {
    deepwellParams.pages = pages
  }

  return expectDeepwellStringArray(
    await requestDeepwell("page_tags_select", deepwellParams),
    "page_tags_select"
  )
}

async function selectPages(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const deepwellParams: DeepwellStringParams & {
    site: string
    pagetype?: string
    categories?: string[]
    tags_any?: string[]
    tags_all?: string[]
    tags_none?: string[]
    parent?: string
    created_by?: string
    rating?: string
    order?: string
  } = { site }

  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "pagetype",
    validatePageSelectType
  )
  addOptionalStringArrayField(deepwellParams, params, "categories", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_any", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_all", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_none", "pages.select")
  addOptionalStringField(deepwellParams, params, "parent")
  addOptionalStringField(deepwellParams, params, "created_by")
  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "rating",
    validatePageSelectRating
  )
  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "order",
    validatePageSelectOrder
  )

  return expectDeepwellStringArray(
    await requestDeepwell("page_select", deepwellParams),
    "page_select"
  )
}

async function requestDeepwell(
  method: string,
  params: Record<string, XmlRpcValue>
): Promise<unknown> {
  try {
    return await client.request(method, params)
  } catch (error) {
    console.error(`Unexpected Deepwell XML-RPC bridge error for ${method}`, error)
    throw new XmlRpcFault(-32603, `XML-RPC Deepwell request failed: ${method}`)
  }
}

function expectParamCount(call: XmlRpcCall, expectedCount: number): void {
  if (call.params.length !== expectedCount) {
    throw new XmlRpcFault(
      -32602,
      `${call.methodName} expects ${expectedCount} parameter${expectedCount === 1 ? "" : "s"}`
    )
  }
}

function getMethodDefinition(methodName: string): MethodDefinition {
  if (!hasMethodDefinition(methodName)) {
    throw new XmlRpcFault(-32601, `Unsupported XML-RPC method: ${methodName}`)
  }
  const definition = METHOD_DEFINITIONS[methodName]
  return definition
}

function getStringParam(call: XmlRpcCall, index: number, name: string): string {
  const value = call.params[index]
  if (typeof value !== "string") {
    throw new XmlRpcFault(-32602, `Expected string parameter: ${name}`)
  }
  return value
}

function getStructParam(
  call: XmlRpcCall,
  index: number,
  name: string
): Record<string, XmlRpcValue> {
  const value = call.params[index]
  if (!isXmlRpcStruct(value)) {
    throw new XmlRpcFault(-32602, `Expected struct parameter: ${name}`)
  }
  return value
}

function getRequiredStructString(
  params: Record<string, XmlRpcValue>,
  name: string
): string {
  const value = params[name]
  if (typeof value !== "string" || value.length === 0) {
    throw new XmlRpcFault(-32602, `Expected string field: ${name}`)
  }
  return value
}

function getOptionalStructStringArray(
  params: Record<string, XmlRpcValue>,
  name: string
): string[] | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    throw new XmlRpcFault(-32602, `Expected string array field: ${name}`)
  }
  return value
}

function getOptionalStructString(
  params: Record<string, XmlRpcValue>,
  name: string
): string | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  if (typeof value !== "string") {
    throw new XmlRpcFault(-32602, `Expected string field: ${name}`)
  }
  return value
}

function addOptionalStringField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string
): void {
  const value = getOptionalStructString(params, name)
  if (value !== null) {
    target[name] = value
  }
}

function addOptionalValidatedStringField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string,
  validate: (value: string) => void
): void {
  const value = getOptionalStructString(params, name)
  if (value !== null) {
    validate(value)
    target[name] = value
  }
}

function addOptionalStringArrayField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string,
  methodName: string
): void {
  const value = getOptionalStructStringArray(params, name)
  if (value !== null) {
    if (value.length > MAX_XML_RPC_FILTER_VALUES) {
      throw new XmlRpcFault(
        -32602,
        `${methodName} ${name} is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
      )
    }
    target[name] = value
  }
}

function validatePageSelectType(value: string): void {
  switch (value.trim().toLowerCase()) {
    case "":
    case "*":
    case "all":
    case "normal":
    case "page":
    case "pages":
    case "hidden":
      return
    default:
      throw new XmlRpcFault(-32602, `Unsupported pages.select pagetype: ${value}`)
  }
}

function validatePageSelectRating(value: string): void {
  const trimmed = value.trim()
  if (trimmed === "") {
    return
  }
  const number = trimmed.replace(/^(>=|<=|!=|==|>|<|=)/, "").trim()
  if (
    number === "" ||
    !PAGE_SELECT_DECIMAL_RATING_PATTERN.test(number) ||
    !Number.isFinite(Number(number))
  ) {
    throw new XmlRpcFault(-32602, `Invalid pages.select rating filter: ${value}`)
  }
}

function validatePageSelectOrder(value: string): void {
  const trimmed = value.trim()
  if (trimmed === "") {
    return
  }

  const parts = trimmed.split(/\s+/)
  if (parts.length > 2) {
    throw new XmlRpcFault(-32602, `Invalid pages.select order expression: ${value}`)
  }

  const field = parts[0]?.toLowerCase()
  switch (field) {
    case "created_at":
    case "created":
    case "updated_at":
    case "updated":
    case "fullname":
    case "full_name":
    case "slug":
    case "name":
    case "title":
    case "rating":
    case "score":
      break
    default:
      throw new XmlRpcFault(-32602, `Unsupported pages.select order field: ${field}`)
  }

  const direction = parts[1]?.toLowerCase()
  if (
    direction !== undefined &&
    direction !== "asc" &&
    direction !== "ascending" &&
    direction !== "desc" &&
    direction !== "descending"
  ) {
    throw new XmlRpcFault(
      -32602,
      `Unsupported pages.select order direction: ${direction}`
    )
  }
}

function getArrayParam(call: XmlRpcCall, index: number, name: string): XmlRpcValue[] {
  const value = call.params[index]
  if (!Array.isArray(value)) {
    throw new XmlRpcFault(-32602, `Expected array parameter: ${name}`)
  }
  return value
}

function isXmlRpcStruct(value: XmlRpcValue): value is Record<string, XmlRpcValue> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
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
