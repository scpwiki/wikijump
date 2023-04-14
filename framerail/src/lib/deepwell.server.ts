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

export async function getSiteFromDomain(domain: string) {
  // TODO add site base domain to deepwell
  return {siteId:1,startSlug:'start'}

  const response = await wjfetch(`${DEEPWELL_ROUTE}/site/fromDomain/${domain}`);
  if (!response.ok) {
    throw new Error("Cannot get site from domain")
  }

  return response.json();
}

export async function getSessionData(cookies) {
  const sessionToken = cookies.get('wikijump_token')
  if (!sessionToken) {
    return null
  }

  const response = await wjfetch(`${DEEPWELL_ROUTE}/auth/session`, { method: 'GET', body: sessionToken })
  if (response.status) {
    // invalid session token
    cookies.delete('wikijump_token')
    return null
  }

  if (!response.ok) {
    throw new Error("Cannot get session data from token");
  }

  return response.json()
}
