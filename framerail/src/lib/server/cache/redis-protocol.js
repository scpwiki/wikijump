const CRLF = "\r\n"

/** @typedef {string | number | null | unknown[]} RedisValue */
/** @typedef {{ value: RedisValue; nextOffset: number }} ParsedRedisResponse */
/** @typedef {{ value: string; nextOffset: number }} ParsedRedisLine */

/** @param {(string | number)[]} parts */
export const encodeRedisCommand = (parts) => {
  let command = `*${parts.length}${CRLF}`
  for (const part of parts) {
    const value = `${part}`
    command += `$${Buffer.byteLength(value, "utf8")}${CRLF}${value}${CRLF}`
  }
  return command
}

/**
 * @param {Buffer} buffer
 * @param {number} offset
 * @returns {ParsedRedisLine | null}
 */
const parseLine = (buffer, offset) => {
  const end = buffer.indexOf(CRLF, offset, "utf8")
  if (end === -1) return null
  return {
    value: buffer.toString("utf8", offset, end),
    nextOffset: end + CRLF.length
  }
}

/**
 * @param {Buffer} buffer
 * @param {number} [offset]
 * @returns {ParsedRedisResponse | null}
 */
export const parseRedisResponse = (buffer, offset = 0) => {
  if (offset >= buffer.length) return null

  const type = String.fromCharCode(buffer[offset])
  const payloadOffset = offset + 1
  const line = parseLine(buffer, payloadOffset)
  if (!line) return null

  if (type === "+") {
    return { value: line.value, nextOffset: line.nextOffset }
  }
  if (type === "-") {
    throw new Error(line.value)
  }
  if (type === ":") {
    return { value: Number.parseInt(line.value, 10), nextOffset: line.nextOffset }
  }
  if (type === "$") {
    const length = Number.parseInt(line.value, 10)
    if (length === -1) {
      return { value: null, nextOffset: line.nextOffset }
    }
    if (!Number.isInteger(length) || length < 0) {
      throw new Error("invalid Redis bulk string length")
    }
    const end = line.nextOffset + length
    const nextOffset = end + CRLF.length
    if (buffer.length < nextOffset) return null
    return {
      value: buffer.toString("utf8", line.nextOffset, end),
      nextOffset
    }
  }
  if (type === "*") {
    const length = Number.parseInt(line.value, 10)
    if (length === -1) {
      return { value: null, nextOffset: line.nextOffset }
    }
    if (!Number.isInteger(length) || length < 0) {
      throw new Error("invalid Redis array length")
    }

    /** @type {RedisValue[]} */
    const values = []
    let nextOffset = line.nextOffset
    for (let index = 0; index < length; index += 1) {
      /** @type {ParsedRedisResponse | null} */
      const parsed = parseRedisResponse(buffer, nextOffset)
      if (!parsed) return null
      values.push(parsed.value)
      nextOffset = parsed.nextOffset
    }

    return { value: values, nextOffset }
  }
  throw new Error("unsupported Redis response type")
}
