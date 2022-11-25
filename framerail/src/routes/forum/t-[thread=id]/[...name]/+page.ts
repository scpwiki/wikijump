import type { PageLoad } from "./$types"

export const load: PageLoad = ({ params }) => {
  return { threadId: parseInt(params.thread, 10) }
}
