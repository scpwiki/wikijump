import { redirect } from '@sveltejs/kit';
import { wjfetch } from '$lib/fetch.ts';
import { pageView } from '$lib/server/deepwell/views.ts';
import type { PageRoute } from '$lib/server/deepwell/views.ts';
import type { Optional } from '$lib/types.ts';

// TODO form single deepwell request that does all the relevant prep stuff here

export async function loadPage(slug: Optional<string>, extra: Optional<string>, request, cookies) {
  const url = new URL(request.url)
  const domain = url.hostname
  const route = (slug || extra) ? { slug, extra } : null;
  const sessionToken = cookies.get('wikijump_token');
  const language = request.headers.get("Accept-Language");

  const view = await pageView(domain, route, sessionToken, language)
  console.log(view)

  return view
}
