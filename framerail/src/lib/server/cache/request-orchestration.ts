interface CachedResponseReaders {
  readPrimary: () => Promise<Response | null>
  readFallback: () => Promise<Response | null>
}

export async function readCachedResponseWithFallback({
  readPrimary,
  readFallback
}: CachedResponseReaders): Promise<Response | null> {
  try {
    const response = await readPrimary()
    if (response) return response
  } catch {
    // Continue to the authoritative backend-backed lookup.
  }

  try {
    return await readFallback()
  } catch {
    return null
  }
}

interface CachedRequestHandlers {
  readCachedResponse: () => Promise<Response | null>
  resolveResponse: () => Promise<Response>
  applySecurityHeaders: (response: Response) => void
  writeResolvedResponse: (response: Response) => Promise<void>
}

export async function handleCachedRequest({
  readCachedResponse,
  resolveResponse,
  applySecurityHeaders,
  writeResolvedResponse
}: CachedRequestHandlers): Promise<Response> {
  const cachedResponse = await readCachedResponse()
  if (cachedResponse) {
    applySecurityHeaders(cachedResponse)
    return cachedResponse
  }

  const response = await resolveResponse()
  applySecurityHeaders(response)
  await writeResolvedResponse(response)
  return response
}
