import { client } from "$lib/server/deepwell/index.ts"

export async function getFileByHash(
  /** Either a Uint8Array or a hex string */
  fileHash: Uint8Array | string
): Promise<Blob> {
  let res = await client.request(
    "blob_get",
    typeof fileHash === "string" ? fileHash : Buffer.from(fileHash).toString("hex")
  )

  return new Blob([new Uint8Array(res.data)], { type: res.mime })
}
