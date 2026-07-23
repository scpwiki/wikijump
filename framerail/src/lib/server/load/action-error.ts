import type { JsonValue } from "$lib/types"

export interface PublicActionError {
  message: string
  code?: number
  data?: JsonValue
}

function isJsonValue(value: unknown): value is JsonValue {
  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return true
  }
  if (Array.isArray(value)) {
    return value.every(isJsonValue)
  }
  if (typeof value === "object") {
    return Object.values(value).every(isJsonValue)
  }
  return false
}

export function normalizeActionError(error: unknown): PublicActionError {
  if (!error || typeof error !== "object") {
    return { message: String(error) }
  }

  const candidate = error as {
    message?: unknown
    code?: unknown
    data?: unknown
  }
  const normalized: PublicActionError = {
    message:
      typeof candidate.message === "string"
        ? candidate.message
        : "An unexpected server error occurred."
  }
  if (typeof candidate.code === "number") {
    normalized.code = candidate.code
  }
  if (isJsonValue(candidate.data)) {
    normalized.data = candidate.data
  }
  return normalized
}
