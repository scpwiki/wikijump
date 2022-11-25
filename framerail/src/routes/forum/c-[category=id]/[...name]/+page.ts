import type { PageLoad } from "./$types"

export const load: PageLoad = ({ params }) => {
  return { categoryId: parseInt(params.category, 10) }
}
