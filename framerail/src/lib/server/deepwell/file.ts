import { client } from "$lib/server/deepwell"

export async function getFileByHash(
  /** Either a Uint8Array or a hex string */
  fileHash: Uint8Array | string
): Promise<Blob> {
  const res = await client.request(
    "blob_get",
    typeof fileHash === "string" ? fileHash : Buffer.from(fileHash).toString("hex")
  )

  return new Blob([new Uint8Array(res.data)], { type: res.mime })
}

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

export async function uploadToPresignUrl(url: string, file: File): Promise<void> {
  const response = await fetch(url, {
    method: "PUT",
    body: file
  })

  if (!response.ok) {
    throw new Error(`Blob upload failed with HTTP status ${response.status}`)
  }
}
