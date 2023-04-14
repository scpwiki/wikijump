import { wjfetch } from '$lib/fetch.ts';
import { getSiteFromDomain, getSessionData } from '$lib/deepwell.server.ts';

function parsePath(path: string) {
  // TODO parse /norender/true etc
  //      see wikidot-path crate
  //      or delegate to deepwell?
}

// TODO form single deepwell request that does all the relevant prep stuff here

export async function processPage(slug, options, request, cookies) {
  const language = request.headers.get("Accept-Language");
  const url = new URL(request.url)

  const site = await getSiteFromDomain(url.hostname)
  if (!slug) slug = site.startSlug
  const session = await getSessionData(cookies)

  return { site, session, slug, options }
}
