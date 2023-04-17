// TODO refactor into proper TS service

import { wjfetch } from '$lib/fetch.ts';

const DEEPWELL_HOST = process.env.DEEPWELL_HOST || 'localhost';
const DEEPWELL_PORT = 2747;
const DEEPWELL_ROUTE = `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/api/trusted`;

export function wellfetch(path, options = {}) {
  if (!path.startsWith('/')) {
    throw new Error(`DEEPWELL path does not start with /: ${path}`);
  }

  const url = `${DEEPWELL_ROUTE}${path}`;
  return wjfetch(url, options)
}

export async function ping(): void {
  const response = await wellfetch('/ping');
  if (!response.ok) {
    throw new Error("Cannot ping DEEPWELL!")
  }
}
