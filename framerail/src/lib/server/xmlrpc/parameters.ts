import { XmlRpcFault, type XmlRpcCall, type XmlRpcValue } from "./protocol"

export type XmlRpcStruct = Record<string, XmlRpcValue>

function stringifyStringOrInteger(value: unknown, errorMessage: string): string {
  if (typeof value === "string") return value
  if (typeof value === "number" && Number.isSafeInteger(value)) return String(value)
  throw new XmlRpcFault(-32602, errorMessage)
}

export function expectParamCount(call: XmlRpcCall, expectedCount: number): void {
  if (call.params.length !== expectedCount) {
    throw new XmlRpcFault(
      -32602,
      `${call.methodName} expects ${expectedCount} parameter${expectedCount === 1 ? "" : "s"}`
    )
  }
}

export function expectUsersGetMeParams(call: XmlRpcCall): void {
  const [value] = call.params
  if (
    call.params.length === 0 ||
    (call.params.length === 1 &&
      ((Array.isArray(value) && value.length === 0) ||
        (isXmlRpcStruct(value) && Object.keys(value).length === 0)))
  ) {
    return
  }

  throw new XmlRpcFault(
    -32602,
    "users.get_me expects no parameters or one empty struct/array parameter"
  )
}

export function getStringParam(call: XmlRpcCall, index: number, name: string): string {
  const value = call.params[index]
  if (typeof value !== "string") {
    throw new XmlRpcFault(-32602, `Expected string parameter: ${name}`)
  }
  return value
}

export function getStructParam(
  call: XmlRpcCall,
  index: number,
  name: string
): XmlRpcStruct {
  const value = call.params[index]
  if (!isXmlRpcStruct(value)) {
    throw new XmlRpcFault(-32602, `Expected struct parameter: ${name}`)
  }
  return value
}

export function getRequiredStructString(params: XmlRpcStruct, name: string): string {
  const value = params[name]
  if (typeof value !== "string" || value.length === 0) {
    throw new XmlRpcFault(-32602, `Expected string field: ${name}`)
  }
  return value
}

export function getRequiredStructStringArray(
  params: XmlRpcStruct,
  name: string
): string[] {
  const value = getOptionalStructStringArray(params, name)
  if (value === null) {
    throw new XmlRpcFault(-32602, `Expected string array field: ${name}`)
  }
  return value
}

export function getRequiredStructStringOrIntArray(
  params: XmlRpcStruct,
  name: string
): string[] {
  const value = params[name]
  if (!Array.isArray(value)) {
    throw new XmlRpcFault(-32602, `Expected string or integer array field: ${name}`)
  }
  return value.map((entry) =>
    stringifyStringOrInteger(entry, `Expected string or integer array field: ${name}`)
  )
}

export function getOptionalStructStringArray(
  params: XmlRpcStruct,
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

export function getOptionalStructString(
  params: XmlRpcStruct,
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

export function getOptionalStructStringOrInt(
  params: XmlRpcStruct,
  name: string
): string | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  return stringifyStringOrInteger(value, `Expected string or integer field: ${name}`)
}

export function getArrayParam(
  call: XmlRpcCall,
  index: number,
  name: string
): XmlRpcValue[] {
  const value = call.params[index]
  if (!Array.isArray(value)) {
    throw new XmlRpcFault(-32602, `Expected array parameter: ${name}`)
  }
  return value
}

export function isXmlRpcStruct(value: unknown): value is XmlRpcStruct {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
