import type { JsonValue } from "$lib/types"
import { fail } from "@sveltejs/kit"

const DEEPWELL_PERMISSION_DENIED = 3106

class InvalidActionRequestError extends Error {}
export class MissingActionSessionError extends Error {}

export class PageActionContextMismatchError extends Error {
  readonly code = DEEPWELL_PERMISSION_DENIED
}

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

export async function readActionJson<T>(request: Request): Promise<T> {
  try {
    return (await request.json()) as T
  } catch {
    throw new InvalidActionRequestError("Invalid JSON request body.")
  }
}

export function failForMissingSession(body: Record<string, unknown> = {}) {
  return fail(401, {
    ...body,
    message: "Authentication required."
  })
}

export function failForActionError(
  error: unknown,
  body: Record<string, unknown> = {},
  fallbackStatus = 500
) {
  const details = normalizeActionError(error)
  const status =
    error instanceof InvalidActionRequestError
      ? 400
      : error instanceof MissingActionSessionError
        ? 401
        : details.code === DEEPWELL_PERMISSION_DENIED
          ? 403
          : fallbackStatus
  return fail(status, {
    ...body,
    ...details
  })
}
