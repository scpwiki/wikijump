// TODO refactor into proper TS service

import { wjfetch } from '$lib/fetch.ts';

const DEEPWELL_HOST = process.env.DEEPWELL_HOST || 'localhost';
const DEEPWELL_PORT = 2747;
const DEEPWELL_ROUTE = `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/api/trusted`;

export async function ping(): void {
  const response = await wjfetch(`${DEEPWELL_ROUTE}/ping`);
  if (!response.ok) {
    throw new Error("Cannot ping DEEPWELL!")
  }
}
