export async function uploadToPresignUrl(url: string, file: File): Promise<void> {
  const response = await fetch(url, {
    method: "PUT",
    body: file
  })

  if (!response.ok) {
    throw new Error(`Blob upload failed with HTTP status ${response.status}`)
  }
}
