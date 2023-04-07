import { ping } from '$lib/deepwell.server.ts';

export async function load() {
  return {
    ping: await ping(),
  };
}
