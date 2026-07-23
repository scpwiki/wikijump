import { getAuthenticatedUser } from "$lib/server/xmlrpc-authentication"
import {
  expectParamCount,
  expectUsersGetMeParams,
  getArrayParam,
  getFileOne,
  getFilesMeta,
  getPageOne,
  getPagesMeta,
  getPosts,
  getStringParam,
  isXmlRpcStruct,
  saveFileOne,
  savePageOne,
  selectCategories,
  selectFiles,
  selectPages,
  selectPosts,
  selectTags
} from "$lib/server/xmlrpc-resource-methods"
import {
  XmlRpcFault,
  type XmlRpcCall,
  type XmlRpcValue
} from "$lib/server/xmlrpc-protocol"

interface MethodDefinition {
  help: string
  signatures: string[][]
}

interface XmlRpcDispatchOptions {
  allowMulticall: boolean
  requestIp: string
}

const MAX_XML_RPC_MULTICALLS = 100
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

export async function dispatchXmlRpcCall(
  call: XmlRpcCall,
  options: XmlRpcDispatchOptions
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
      return dispatchMulticall(call, options.requestIp)
    case "categories.select":
      expectParamCount(call, 1)
      return selectCategories(call)
    case "tags.select":
      expectParamCount(call, 1)
      return selectTags(call, options.requestIp)
    case "pages.select":
      expectParamCount(call, 1)
      return selectPages(call)
    case "pages.get_meta":
      expectParamCount(call, 1)
      return getPagesMeta(call, options.requestIp)
    case "pages.get_one":
      expectParamCount(call, 1)
      return getPageOne(call, options.requestIp)
    case "pages.save_one":
      expectParamCount(call, 1)
      return savePageOne(call, options.requestIp)
    case "files.select":
      expectParamCount(call, 1)
      return selectFiles(call)
    case "files.get_meta":
      expectParamCount(call, 1)
      return getFilesMeta(call)
    case "files.get_one":
      expectParamCount(call, 1)
      return getFileOne(call)
    case "files.save_one":
      expectParamCount(call, 1)
      return saveFileOne(call, options.requestIp)
    case "users.get_me":
      expectUsersGetMeParams(call)
      return getAuthenticatedUser(options.requestIp)
    case "posts.select":
      expectParamCount(call, 1)
      return selectPosts(call)
    case "posts.get":
      expectParamCount(call, 1)
      return getPosts(call)
    default:
      throw new XmlRpcFault(-32601, `Unsupported XML-RPC method: ${call.methodName}`)
  }
}

async function dispatchMulticall(
  call: XmlRpcCall,
  requestIp: string
): Promise<XmlRpcValue[]> {
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
        { allowMulticall: false, requestIp }
      )
      results.push([value])
    } catch (error) {
      const fault =
        error instanceof XmlRpcFault
          ? error
          : new XmlRpcFault(-32603, "system.multicall child call failed")
      if (!(error instanceof XmlRpcFault)) {
        console.error("Unexpected system.multicall child error", error)
      }
      results.push({ faultCode: fault.faultCode, faultString: fault.faultString })
    }
  }
  return results
}

function getMethodDefinition(methodName: string): MethodDefinition {
  if (!Object.prototype.hasOwnProperty.call(METHOD_DEFINITIONS, methodName)) {
    throw new XmlRpcFault(-32601, `Unsupported XML-RPC method: ${methodName}`)
  }
  const definition = METHOD_DEFINITIONS[methodName]
  return definition
}
