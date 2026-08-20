import { client } from "$lib/server/deepwell"

export async function startBlobUpload(userId: number, blobSize: number) {
  return await client.request("blob_upload", {
    user_id: userId,
    blob_size: blobSize
  })
}

export async function cancelBlobUpload(userId: number, pendingBlobId: string) {
  return await client.request("blob_cancel", {
    user_id: userId,
    pending_blob_id: pendingBlobId
  })
}

export async function uploadToPresignUrl(url: string, file: File) {
  return await fetch(url, {
    method: "PUT",
    body: file
  })
}
