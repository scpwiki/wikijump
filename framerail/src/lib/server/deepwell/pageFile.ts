import defaults from "$lib/defaults"

import { client } from "$lib/server/deepwell"
import { startBlobUpload, uploadToPresignUrl } from "$lib/server/deepwell/file"

import type { FileRevisionModel, FileRevisionType, Nullable, Optional } from "$lib/types"

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
  deleted: Optional<boolean>
): Promise<PageFile[]> {
  return client.request("page_get_files", {
    site_id: siteId,
    page_id: pageId,
    deleted
  })
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
export async function pageFileCreate(
  siteId: number,
  pageId: number,
  userId: number,
  ipAddress: string,
  name: Optional<string>,
  file: File,
  revisionComments: Optional<string>,
  bypass_filter = false
): Promise<PageFileCreate> {
  const presign = await startBlobUpload(userId, file.size)
  await uploadToPresignUrl(presign.presign_url, file)

  return await client.request("file_create", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    ip_address: ipAddress,
    name: name ?? file.name,
    uploaded_blob_id: presign.pending_blob_id,
    revision_comments: revisionComments ?? "",
    bypass_filter: bypass_filter
  })
}

/* ----- Page File Delete ----- */
export interface PageFileDelete {
  file_id: number
  file_revision_id: number
  file_revision_number: number
}
export async function pageFileDelete(
  siteId: number,
  pageId: number,
  userId: number,
  fileId: number,
  lastRevisionId: number,
  revisionComments: string
): Promise<PageFileDelete> {
  return await client.request("file_delete", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    file: fileId,
    last_revision_id: lastRevisionId,
    revision_comments: revisionComments
  })
}

/* ----- Page File Edit ----- */
export async function pageFileEdit(
  siteId: number,
  pageId: number,
  userId: number,
  fileId: number,
  name: Optional<string>,
  file: Optional<File>,
  lastRevisionId: number,
  revisionComments: Optional<string>,
  bypassFilter = false
): Promise<PageFileCreate> {
  let presignId
  if (file && file instanceof File) {
    const presign = await startBlobUpload(userId, file.size)
    await uploadToPresignUrl(presign.presign_url, file)
    presignId = presign.pending_blob_id
  }

  return await client.request("file_edit", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    file_id: fileId,
    last_revision_id: lastRevisionId,
    name,
    uploaded_blob_id: presignId,
    revision_comments: revisionComments,
    bypass_filter: bypassFilter
  })
}

/* ----- Page File Move ----- */
export async function pageFileMove(
  siteId: number,
  currentPageId: number,
  destinationPage: string | number,
  userId: number,
  fileId: number,
  lastRevisionId: number,
  name: Optional<string>,
  revisionComments: Optional<string>
): Promise<PageFileCreate> {
  return await client.request("file_move", {
    site_id: siteId,
    current_page_id: currentPageId,
    destination_page: destinationPage,
    user_id: userId,
    file_id: fileId,
    last_revision_id: lastRevisionId,
    name,
    revision_comments: revisionComments
  })
}

/* ----- Page File Restore ----- */
interface PageFileRestore {
  page_id: number
  file_id: number
  name: string
  file_revision_id: number
  file_revision_number: number
}
export async function pageFileRestore(
  siteId: number,
  pageId: number,
  userId: number,
  fileId: number,
  newPage: Optional<string | number>,
  newName: Optional<string>,
  revisionComments: string
): Promise<PageFileRestore> {
  return client.request("file_restore", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    file_id: fileId,
    new_page: newPage,
    new_name: newName,
    revision_comments: revisionComments
  })
}

/* ----- Page File History ----- */
export async function pageFileHistory(
  siteId: number,
  pageId: Optional<number>,
  fileId: number,
  revisionNumber: Optional<number>,
  limit: Optional<number>
): Promise<FileRevisionModel[]> {
  return client.request("file_revision_range", {
    site_id: siteId,
    page_id: pageId,
    file_id: fileId,
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
    revision_direction: "before",
    limit: limit ?? defaults.page.history.limit
  })
}

/* ----- Page File Rollback ----- */
export async function pageFileRollback(
  siteId: number,
  pageId: number,
  userId: number,
  fileId: number,
  lastRevisionId: number,
  revisionNumber: number,
  revisionComments: Optional<string>,
  bypassFilter = false
): Promise<Nullable<PageFile>> {
  return client.request("file_rollback", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    file: fileId,
    last_revision_id: lastRevisionId,
    revision_number: revisionNumber,
    revision_comments: revisionComments,
    bypass_filter: bypassFilter
  })
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
