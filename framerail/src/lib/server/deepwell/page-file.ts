import defaults from "$lib/defaults"

import { client } from "$lib/server/deepwell"
import { startBlobUpload, uploadToPresignUrl } from "$lib/server/deepwell/file"
import { pageEditPermission } from "$lib/server/deepwell/page"
import {
  buildPageFileCreatePayload,
  buildPageFileEditPayload,
  buildPageFileRestorePayload,
  buildPageFileRollbackPayload
} from "$lib/server/deepwell/page-file-mutation-payloads"

import type { FileRevisionModel, FileRevisionType, Nullable, Optional } from "$lib/types"
import type { RequestContext } from "./request-context"

const DEEPWELL_PERMISSION_DENIED = 3106

/* ----- Common Interface ----- */
export interface PageFile {
  file_id: number
  file_created_at: string
  file_updated_at: Nullable<string>
  file_deleted_at: Nullable<string>
  page_id: number
  revision_id: number
  revision_type: FileRevisionType
  revision_created_at: string
  revision_number: number
  revision_user_id: number
  name: string
  data: Nullable<string>
  mime: string
  size: number
  s3_hash: string
  revision_comments: string
  hidden_fields: string[]
}

/* ----- Page File List ----- */
export async function pageFileList(
  siteId: number,
  pageId: number,
  deleted: Optional<boolean>,
  requestContext: RequestContext
): Promise<PageFile[]> {
  return client.request(
    "page_get_files",
    {
      site_id: siteId,
      page_id: pageId,
      deleted
    },
    requestContext
  )
}

/* ----- Page File Get ----- */
// Not used for now
export async function pageFileGet(
  siteId: number,
  pageId: number,
  userId: number,
  fileId: number,
  /** Also request the contents of the file */
  data: boolean
): Promise<Nullable<PageFile>> {
  return await client.request("file_get", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    file: fileId,
    data
  })
}

/* ----- Page File Create ----- */
interface PageFileCreate {
  file_id: number
  file_revision_id: number
  blob_created: boolean
}
export interface PageFileCreateInput {
  siteId: number
  pageId: number
  userId: number
  name: Optional<string>
  file: File
  revisionComments: Optional<string>
  ipAddress: string
  bypassFilter?: boolean
}
export async function pageFileCreate(
  input: PageFileCreateInput,
  requestContext: RequestContext
): Promise<PageFileCreate> {
  const {
    siteId,
    pageId,
    userId,
    name,
    file,
    revisionComments,
    ipAddress,
    bypassFilter = false
  } = input
  await requireFileParentEditPermission(requestContext)
  const presign = await startBlobUpload(userId, file.size)
  await uploadToPresignUrl(presign.presign_url, file)

  return await client.request(
    "file_create",
    buildPageFileCreatePayload({
      siteId,
      pageId,
      userId,
      name: name ?? file.name,
      pendingBlobId: presign.pending_blob_id,
      revisionComments: revisionComments ?? "",
      ipAddress,
      bypassFilter
    }),
    requestContext
  )
}

/* ----- Page File Delete ----- */
export interface PageFileDelete {
  file_id: number
  file_revision_id: number
  file_revision_number: number
}
export interface PageFileDeleteInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  lastRevisionId: number
  revisionComments: string
}
export async function pageFileDelete(
  input: PageFileDeleteInput,
  requestContext: RequestContext
): Promise<PageFileDelete> {
  const { siteId, pageId, userId, fileId, lastRevisionId, revisionComments } = input
  return await client.request(
    "file_delete",
    {
      site_id: siteId,
      page_id: pageId,
      user_id: userId,
      file: fileId,
      last_revision_id: lastRevisionId,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page File Edit ----- */
export interface PageFileEditInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  name: Optional<string>
  file: Optional<File>
  lastRevisionId: number
  revisionComments: Optional<string>
  ipAddress: string
  bypassFilter?: boolean
}
export async function pageFileEdit(
  input: PageFileEditInput,
  requestContext: RequestContext
): Promise<PageFileCreate> {
  const {
    siteId,
    pageId,
    userId,
    fileId,
    name,
    file,
    lastRevisionId,
    revisionComments,
    ipAddress,
    bypassFilter = false
  } = input
  let presignId: Optional<string>
  if (file && file instanceof File) {
    await requireFileParentEditPermission(requestContext)
    const presign = await startBlobUpload(userId, file.size)
    await uploadToPresignUrl(presign.presign_url, file)
    presignId = presign.pending_blob_id
  }

  return await client.request(
    "file_edit",
    buildPageFileEditPayload({
      siteId,
      pageId,
      userId,
      fileId,
      lastRevisionId,
      name,
      pendingBlobId: presignId,
      revisionComments,
      ipAddress,
      bypassFilter
    }),
    requestContext
  )
}

async function requireFileParentEditPermission(
  requestContext: RequestContext
): Promise<void> {
  const permission = await pageEditPermission(requestContext)
  if (!permission.can_edit) {
    const error = new Error("Permission denied.") as Error & { code: number }
    error.code = DEEPWELL_PERMISSION_DENIED
    throw error
  }
}

/* ----- Page File Move ----- */
export interface PageFileMoveInput {
  siteId: number
  currentPageId: number
  destinationPage: string | number
  userId: number
  fileId: number
  lastRevisionId: number
  name: Optional<string>
  revisionComments: Optional<string>
}
export async function pageFileMove(
  input: PageFileMoveInput,
  requestContext: RequestContext
): Promise<PageFileCreate> {
  const {
    siteId,
    currentPageId,
    destinationPage,
    userId,
    fileId,
    lastRevisionId,
    name,
    revisionComments
  } = input
  return await client.request(
    "file_move",
    {
      site_id: siteId,
      current_page_id: currentPageId,
      destination_page: destinationPage,
      user_id: userId,
      file_id: fileId,
      last_revision_id: lastRevisionId,
      name,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page File Restore ----- */
interface PageFileRestore {
  page_id: number
  file_id: number
  name: string
  file_revision_id: number
  file_revision_number: number
}
export interface PageFileRestoreInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  newPage: Optional<string | number>
  newName: Optional<string>
  revisionComments: string
  ipAddress: string
  bypassFilter?: boolean
}
export async function pageFileRestore(
  input: PageFileRestoreInput,
  requestContext: RequestContext
): Promise<PageFileRestore> {
  const {
    siteId,
    pageId,
    userId,
    fileId,
    newPage,
    newName,
    revisionComments,
    ipAddress,
    bypassFilter = false
  } = input
  return client.request(
    "file_restore",
    buildPageFileRestorePayload({
      siteId,
      pageId,
      userId,
      fileId,
      newPage,
      newName,
      revisionComments,
      ipAddress,
      bypassFilter
    }),
    requestContext
  )
}

/* ----- Page File History ----- */
export async function pageFileHistory(
  siteId: number,
  pageId: Optional<number>,
  fileId: number,
  revisionNumber: Optional<number>,
  limit: Optional<number>,
  requestContext: RequestContext
): Promise<FileRevisionModel[]> {
  return client.request(
    "file_revision_range",
    {
      site_id: siteId,
      page_id: pageId,
      file_id: fileId,
      revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
      revision_direction: "before",
      limit: limit ?? defaults.page.history.limit
    },
    requestContext
  )
}

/* ----- Page File Rollback ----- */
export interface PageFileRollbackInput {
  siteId: number
  pageId: number
  userId: number
  fileId: number
  lastRevisionId: number
  revisionNumber: number
  revisionComments: Optional<string>
  ipAddress: string
  bypassFilter?: boolean
}
export async function pageFileRollback(
  input: PageFileRollbackInput,
  requestContext: RequestContext
): Promise<Nullable<PageFile>> {
  const {
    siteId,
    pageId,
    userId,
    fileId,
    lastRevisionId,
    revisionNumber,
    revisionComments,
    ipAddress,
    bypassFilter = false
  } = input
  return client.request(
    "file_rollback",
    buildPageFileRollbackPayload({
      siteId,
      pageId,
      userId,
      fileId,
      lastRevisionId,
      revisionNumber,
      revisionComments,
      ipAddress,
      bypassFilter
    }),
    requestContext
  )
}

/* ----- Page File Revision ----- */
// Not used for now
export async function pageFileRevision(
  siteId: number,
  pageId: Optional<number>,
  fileId: number,
  revisionNumber: Optional<number>
): Promise<Nullable<FileRevisionModel>> {
  return client.request("file_revision_get", {
    site_id: siteId,
    page_id: pageId,
    file_id: fileId,
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber
  })
}
