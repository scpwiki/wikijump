export const isHeaderPair = (value) => {
  return (
    Array.isArray(value) &&
    value.length === 2 &&
    typeof value[0] === "string" &&
    typeof value[1] === "string"
  )
}

const isCachedArticleResponse = (value) => {
  return (
    value !== null &&
    typeof value === "object" &&
    Number.isInteger(value.status) &&
    value.status >= 200 &&
    value.status <= 599 &&
    Array.isArray(value.headers) &&
    value.headers.every(isHeaderPair) &&
    typeof value.body === "string"
  )
}

export const normalizeCachedArticleResponseEntry = (value) => {
  if (!isCachedArticleResponse(value)) return null

  return {
    status: value.status,
    headers: value.headers.map(([name, headerValue]) => [name, headerValue]),
    body: value.body
  }
}

export const copyCachedArticleResponseEntry = (entry) => {
  const copy = {
    status: entry.status,
    headers: entry.headers.map(([name, value]) => [name, value]),
    body: entry.body
  }
  if (Buffer.isBuffer(entry.bodyBuffer)) {
    copy.bodyBuffer = Buffer.from(entry.bodyBuffer)
  }
  return copy
}
