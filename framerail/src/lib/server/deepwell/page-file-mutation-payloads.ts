interface PageFileCreatePayloadInput {
  siteId: number
  pageId: number
  userId: number
  name: string
  pendingBlobId: string
  revisionComments: string
  ipAddress: string
  bypassFilter: boolean
}

interface PageFileEditPayloadInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  lastRevisionId: number
  name: string | undefined
  pendingBlobId: string | undefined
  revisionComments: string | undefined
  ipAddress: string
  bypassFilter: boolean
}

interface PageFileRestorePayloadInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  newPage: string | number | undefined
  newName: string | undefined
  revisionComments: string
  ipAddress: string
  bypassFilter: boolean
}

interface PageFileRollbackPayloadInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  lastRevisionId: number
  revisionNumber: number
  revisionComments: string | undefined
  ipAddress: string
  bypassFilter: boolean
}

export function withPageFileClientAddress<T>(
  getClientAddress: () => string,
  input: T
): T & { ipAddress: string } {
  return {
    ...input,
    ipAddress: getClientAddress()
  }
}

export function buildPageFileCreatePayload(input: PageFileCreatePayloadInput) {
  return {
    site_id: input.siteId,
    page_id: input.pageId,
    user_id: input.userId,
    name: input.name,
    uploaded_blob_id: input.pendingBlobId,
    revision_comments: input.revisionComments,
    ip_address: input.ipAddress,
    bypass_filter: input.bypassFilter
  }
}

export function buildPageFileEditPayload(input: PageFileEditPayloadInput) {
  return {
    site_id: input.siteId,
    page_id: input.pageId,
    user_id: input.userId,
    file_id: input.fileId,
    last_revision_id: input.lastRevisionId,
    name: input.name,
    uploaded_blob_id: input.pendingBlobId,
    revision_comments: input.revisionComments,
    ip_address: input.ipAddress,
    bypass_filter: input.bypassFilter
  }
}

export function buildPageFileRestorePayload(input: PageFileRestorePayloadInput) {
  return {
    site_id: input.siteId,
    page_id: input.pageId,
    user_id: input.userId,
    file_id: input.fileId,
    new_page: input.newPage,
    new_name: input.newName,
    revision_comments: input.revisionComments,
    ip_address: input.ipAddress,
    bypass_filter: input.bypassFilter
  }
}

export function buildPageFileRollbackPayload(input: PageFileRollbackPayloadInput) {
  return {
    site_id: input.siteId,
    page_id: input.pageId,
    user_id: input.userId,
    file: input.fileId,
    last_revision_id: input.lastRevisionId,
    revision_number: input.revisionNumber,
    revision_comments: input.revisionComments,
    ip_address: input.ipAddress,
    bypass_filter: input.bypassFilter
  }
}
