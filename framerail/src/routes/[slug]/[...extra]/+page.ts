/** @type {import("./$types").PageLoad} */
export function load({ params }) {
  return {
    slug: params.slug,
    options: params.extra // TODO parse
  }
}
