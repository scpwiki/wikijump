const AJAX_MODULE_CONNECTOR_HEADERS = {
  "cache-control": "no-store",
  "content-type": "application/json; charset=utf-8"
}
const MAX_AJAX_MODULE_CONNECTOR_BODY_BYTES = 131_072
const CONTROL_FIELDS = new Set([
  "moduleName",
  "module_body",
  "wikidot_token7",
  "callbackIndex",
  "eventSource"
])

const jsonResponse = (body, status = 200, extraHeaders = {}) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { ...AJAX_MODULE_CONNECTOR_HEADERS, ...extraHeaders }
  })

const readUrlEncodedForm = async (request) => {
  const contentType = request.headers.get("content-type")?.split(";", 1)[0].trim()
  if (contentType !== "application/x-www-form-urlencoded") {
    throw new TypeError("AJAX Module Connector requires URL-encoded form data")
  }

  const contentLength = request.headers.get("content-length")
  if (contentLength !== null) {
    const normalized = contentLength.trim()
    if (!/^\d+$/.test(normalized)) {
      throw new TypeError("AJAX Module Connector content length is invalid")
    }
    if (Number.parseInt(normalized, 10) > MAX_AJAX_MODULE_CONNECTOR_BODY_BYTES) {
      throw new RangeError("AJAX Module Connector request body is too large")
    }
  }

  const reader = request.body?.getReader()
  if (reader === undefined) {
    return new Map()
  }

  const chunks = []
  let byteLength = 0
  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      byteLength += value.byteLength
      if (byteLength > MAX_AJAX_MODULE_CONNECTOR_BODY_BYTES) {
        throw new RangeError("AJAX Module Connector request body is too large")
      }
      chunks.push(value)
    }
  } finally {
    reader.releaseLock()
  }

  const bytes = new Uint8Array(byteLength)
  let offset = 0
  for (const chunk of chunks) {
    bytes.set(chunk, offset)
    offset += chunk.byteLength
  }
  const body = new TextDecoder("utf-8", { fatal: true }).decode(bytes)
  const form = new URLSearchParams(body)
  const values = new Map()
  for (const [key, value] of form) {
    if (values.has(key)) {
      throw new TypeError(`AJAX Module Connector field is duplicated: ${key}`)
    }
    values.set(key, value)
  }
  return values
}

export const handleAjaxModuleConnectorRequest = async (
  request,
  { siteId, renderListPages }
) => {
  if (request.method !== "POST") {
    return jsonResponse(
      { status: "not_ok", message: "AJAX Module Connector requires POST" },
      405,
      { allow: "POST" }
    )
  }

  let fields
  try {
    fields = await readUrlEncodedForm(request)
  } catch (error) {
    const status = error instanceof RangeError ? 413 : 400
    return jsonResponse(
      {
        status: "not_ok",
        message:
          error instanceof Error
            ? error.message
            : "Malformed AJAX Module Connector request"
      },
      status
    )
  }

  const moduleName = fields.get("moduleName")
  if (moduleName !== "list/ListPagesModule") {
    return jsonResponse({
      status: "not_ok",
      message: `Unsupported AJAX module: ${moduleName ?? ""}`
    })
  }

  const moduleBody = fields.get("module_body")
  if (moduleBody === undefined) {
    return jsonResponse({
      status: "not_ok",
      message: "ListPages module_body is required"
    })
  }

  const parameters = {}
  for (const [key, value] of fields) {
    if (!CONTROL_FIELDS.has(key)) parameters[key] = value
  }

  try {
    const output = await renderListPages({
      siteId,
      moduleBody,
      parameters
    })
    return jsonResponse({ status: "ok", body: output.body })
  } catch (error) {
    console.error("AJAX ListPages rendering failed", error)
    return jsonResponse({
      status: "not_ok",
      message: "Unable to render ListPages module"
    })
  }
}
