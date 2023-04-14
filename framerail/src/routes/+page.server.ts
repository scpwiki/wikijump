import { ping } from '$lib/deepwell.server.ts';
import { processPage } from '$lib/process-page.server.ts';

export async function load({ request, cookies }) {
  // fails if deepwell does
  await ping()

  const { site, session, options } = await processPage(null, null, request, cookies)

  const headers = {};
  for (const entry of request.headers.entries()) {
    const [header, value] = entry;
    headers[header] = value;
  }

  return {
    url: request.url,
    headers,
    site,
    session,
    options,
  }
}
