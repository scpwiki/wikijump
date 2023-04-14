import { ping } from '$lib/deepwell.server.ts';

export async function load() {
  // fails if deepwell does
  await ping()

  return {}
}
