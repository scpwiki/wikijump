const CAPTURE_SIDES = new Set(["source", "local"]);
const WEB_PROTOCOLS = new Set(["http:", "https:"]);

export class BrowserCaptureUrlPolicyError extends Error {}

// Inventory data chooses paths, not network authorities. Live captures stay on
// the selected Wikidot host and local captures stay on the selected Wikijump
// host. Both web schemes are retained because some live Wikidot sites still
// redirect between HTTP and HTTPS on the same host.

function isDomainOrSubdomain(hostname, domain) {
  return hostname === domain || hostname.endsWith(`.${domain}`);
}

function parseWebUrl(rawUrl, side, label) {
  let url;
  try {
    url = new URL(rawUrl);
  } catch {
    throw new BrowserCaptureUrlPolicyError(
      `${side} ${label} must be an absolute HTTP(S) URL`,
    );
  }
  if (!WEB_PROTOCOLS.has(url.protocol)) {
    throw new BrowserCaptureUrlPolicyError(
      `${side} ${label} must use http: or https:`,
    );
  }
  if (url.username || url.password) {
    throw new BrowserCaptureUrlPolicyError(
      `${side} ${label} must not contain credentials`,
    );
  }
  return url;
}

function assertInitialHost(side, url) {
  if (url.port) {
    throw new BrowserCaptureUrlPolicyError(
      `${side} URL must use the default port`,
    );
  }
  if (side === "source" && !isDomainOrSubdomain(url.hostname, "wikidot.com")) {
    throw new BrowserCaptureUrlPolicyError(
      "source URL host must be wikidot.com or a subdomain",
    );
  }
  if (side === "local" && !isDomainOrSubdomain(url.hostname, "wikijump.localhost")) {
    throw new BrowserCaptureUrlPolicyError(
      "local URL host must be wikijump.localhost or a subdomain",
    );
  }
}

export function createBrowserCaptureUrlPolicy(side, rawUrl) {
  if (!CAPTURE_SIDES.has(side)) {
    throw new BrowserCaptureUrlPolicyError(
      "capture side must be source or local",
    );
  }
  const url = parseWebUrl(rawUrl, side, "URL");
  assertInitialHost(side, url);
  const allowedOrigins = Object.freeze([
    `http://${url.hostname}`,
    `https://${url.hostname}`,
  ]);
  return Object.freeze({side, origin: url.origin, allowedOrigins});
}

export function assertBrowserCaptureUrl(policy, rawUrl, label = "URL") {
  const url = parseWebUrl(rawUrl, policy.side, label);
  if (!policy.allowedOrigins.includes(url.origin)) {
    throw new BrowserCaptureUrlPolicyError(
      `${policy.side} ${label} origin is not allowlisted: ${url.origin}`,
    );
  }
  return url;
}

function isMainFrameNavigation(page, request) {
  try {
    return request.isNavigationRequest() && request.frame() === page.mainFrame();
  } catch {
    return false;
  }
}

// This guard owns inventory-selected main navigation only. Subresources remain
// browser-visible parity surface; their cross-origin frame contents are excluded
// by the capture layer rather than pretending this is a DNS-aware network sandbox.
export async function guardMainFrameNavigation(page, policy, onBlocked) {
  if (typeof page.route !== "function") return async () => {};

  const handler = async (route) => {
    const request = route.request();
    try {
      if (isMainFrameNavigation(page, request)) {
        assertBrowserCaptureUrl(policy, request.url(), "navigation URL");
      }
    } catch (error) {
      onBlocked(error);
      await route.abort("blockedbyclient");
      return;
    }
    await route.continue();
  };

  await page.route("**/*", handler);
  return async () => {
    if (typeof page.unroute === "function") {
      await page.unroute("**/*", handler).catch(() => {});
    }
  };
}
