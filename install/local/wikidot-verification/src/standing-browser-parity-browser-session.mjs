import { randomUUID } from "node:crypto";
import dns from "node:dns/promises";
import fs from "node:fs/promises";
import { createRequire } from "node:module";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_REQUEST_INTERVAL_MS,
  acquireBrowserCaptureLock,
  createPersistentBrowserRequestGate,
  createBrowserResponseCache,
  evidenceReplaySubresourceFulfillment,
  fetchBrowserEvidenceResponse,
  isWikidotCapturePublicOrigin,
  installBrowserRequestGate,
  lookupBrowserEvidenceResponse,
  resolveEvidenceReplaySubresourceRedirect,
} from "./browser-request-gate.mjs";
import { startCaptureEgressProxy } from "./capture-egress-proxy.mjs";
import {
  requireExactHttpsOrigins,
  readJsonObject,
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
  sealJsonNoReplace,
  sha256File,
} from "./standing-browser-parity-util.mjs";

const SOURCE_DIR = path.dirname(fileURLToPath(import.meta.url));
export const DEFAULT_PARITY_BROWSER_ROOT = path.resolve(
  SOURCE_DIR,
  "../../../..",
  "framerail",
);
const THROTTLE_CONFIG_SCHEMA = "wikijump.standing_browser_throttle_config.v1";
const MAX_CANDIDATE_FILE_REDIRECTS = 10;
const REDIRECT_STATUSES = new Set([301, 302, 303, 307, 308]);
const PARITY_BROWSER_LAUNCH_ARGS = ["--disable-http-cache"];

export function parityBrowserLaunchOptions(executable) {
  return {executablePath: executable, args: [...PARITY_BROWSER_LAUNCH_ARGS]};
}

export function parityBrowserExecutionMode(mode) {
  if (mode === "live-reference") return "live";
  if (mode === "candidate" || mode === "candidate-case" || mode === "standing") return "candidate";
  throw new Error(`unsupported parity browser mode: ${mode}`);
}

export function parityBrowserRequestIntervalMs(mode) {
  return parityBrowserExecutionMode(mode) === "live"
    ? DEFAULT_REQUEST_INTERVAL_MS
    : 0;
}

function requirePlaywright(browserRoot) {
  const requireFromRoot = createRequire(path.join(browserRoot, "package.json"));
  try {
    return requireFromRoot("playwright");
  } catch (firstError) {
    try {
      return requireFromRoot("@playwright/test");
    } catch (secondError) {
      throw new Error(
        `could not load playwright from ${browserRoot}; pass --browser-root pointing at an installed browser adapter (${firstError.message}; ${secondError.message})`,
      );
    }
  }
}

async function resolveBrowserExecutable(chromium, browserExecutable) {
  const executable = await fs.realpath(
    browserExecutable ?? chromium.executablePath(),
  );
  const stat = await fs.lstat(executable);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error("browser executable must resolve to a regular file");
  }
  return executable;
}

function localConnectLookup(address, allowedOrigins, fallback = dns.lookup) {
  const family = net.isIP(address);
  if (!family)
    throw new Error(
      "candidate local connect address must be a literal IP address",
    );
  const hostnames = new Set(
    allowedOrigins.map((origin) => new URL(origin).hostname.toLowerCase()),
  );
  return async (hostname, options) => {
    if (hostnames.has(String(hostname).toLowerCase())) {
      return [{ address, family }];
    }
    return await fallback(hostname, options);
  };
}

export async function installCandidateFilePortRoute(
  context,
  localOrigins,
  {
    sourceRequestGate = null,
    responseCache = null,
    allowDefaultHttpsPort = false,
  } = {},
) {
  if (!Array.isArray(localOrigins) || localOrigins.length !== 2) {
    throw new Error(
      "candidate local origins must contain exactly page and file origins",
    );
  }
  const origins = localOrigins.map((origin) => new URL(origin));
  const page = origins.find((origin) =>
    origin.hostname.endsWith(".wikijump.localhost"),
  );
  const files = origins.find((origin) =>
    origin.hostname.endsWith(".wjfiles.localhost"),
  );
  if (!page || !files) {
    throw new Error(
      "candidate local origins must contain page and file origins",
    );
  }
  const pageSite = page.hostname.slice(0, -".wikijump.localhost".length);
  const filesSite = files.hostname.slice(0, -".wjfiles.localhost".length);
  if (!pageSite || pageSite !== filesSite) {
    throw new Error(
      "candidate page and file origins must use the same site slug",
    );
  }
  const explicitCandidatePort =
    page.port && page.port !== "443" && files.port === page.port;
  const canonicalStandingPort =
    allowDefaultHttpsPort && !page.port && !files.port;
  if (
    page.protocol !== "https:" ||
    files.protocol !== "https:" ||
    (!explicitCandidatePort && !canonicalStandingPort)
  ) {
    throw new Error(
      allowDefaultHttpsPort
        ? "local page and file origins must use the same explicit non-443 port or canonical HTTPS port"
        : "candidate page and file origins must use the same explicit non-443 port",
    );
  }
  const canonicalFilesOrigin = `https://${files.hostname}`;
  const fileRouteHandler = async (route) => {
    const requestUrl = new URL(route.request().url());
    const isSourceFileAuthority =
      requestUrl.origin === canonicalFilesOrigin || requestUrl.origin === files.origin;
    const isLocalPageFile =
      requestUrl.origin === page.origin &&
      (requestUrl.pathname.startsWith("/local--files/") ||
        requestUrl.pathname.startsWith("/local--code/"));
    if (!isSourceFileAuthority && !isLocalPageFile) {
      await route.fallback();
      return;
    }
    if (requestUrl.origin === canonicalFilesOrigin && files.port) requestUrl.port = files.port;
    const sourcePath = requestUrl.pathname;
    let response;
    for (let redirects = 0; ; redirects += 1) {
      response = await route.fetch({
        url: requestUrl.href,
        maxRedirects: 0,
      });
      if (
        route.request().method?.() !== "GET" ||
        !REDIRECT_STATUSES.has(response.status())
      ) {
        break;
      }
      const location = response.headers().location;
      if (!location) break;
      const redirectUrl = new URL(location, requestUrl);
      if (
        redirectUrl.username ||
        redirectUrl.password ||
        !new Set([canonicalFilesOrigin, files.origin]).has(redirectUrl.origin)
      ) {
        break;
      }
      if (redirects >= MAX_CANDIDATE_FILE_REDIRECTS) {
        throw new Error("candidate file redirect limit exceeded");
      }
      redirectUrl.port = files.port;
      requestUrl.href = redirectUrl.href;
    }
    let responseStatus =
      typeof response.status === "function" ? response.status() : response.status;
    let replayedResponse = false;
    if (
      responseCache !== null &&
      responseStatus !== 200 &&
      (sourcePath.startsWith("/local--files/") || sourcePath.startsWith("/local--code/"))
    ) {
      const cacheUrl = `https://${filesSite}.wdfiles.com${sourcePath}${new URL(route.request().url()).search}`;
      const cached = await lookupBrowserEvidenceResponse({
        responseCache,
        request: route.request(),
        targetUrl: cacheUrl,
      });
      if (cached !== null) {
        response = cached;
        responseStatus = cached.status;
        replayedResponse = true;
      }
    }
    const responseHeaders =
      typeof response.headers === "function" ? response.headers() : response.headers;
    const location = REDIRECT_STATUSES.has(responseStatus)
      ? responseHeaders.location
      : null;
    const redirectUrl = location ? new URL(location, requestUrl) : null;
    const returnsGatedPublicRedirect =
      redirectUrl !== null &&
      !new Set([canonicalFilesOrigin, files.origin]).has(
        redirectUrl.origin,
      ) &&
      isWikidotCapturePublicOrigin(
        redirectUrl,
        route.request().resourceType?.() ?? "other",
        "GET",
      );

    if (
      sourceRequestGate !== null &&
      isSourceFileAuthority &&
      route.request().method?.() === "GET"
    ) {

      if (sourcePath.startsWith("/local--files/")) {
        // Wikidot-rendered page-owned file URLs first hit
        // <site>.wikidot.com/local--files/... and then the corresponding
        // wdfiles authority. The local candidate mirror collapses those public
        // stages into one exempt wjfiles request. Record the two source-side
        // admissions without waiting on the external throttle/Retry-After state
        // before exposing a mirror hit to Chromium. If the mirror falls back to
        // a public redirect, only synthesize the first stage; the actual
        // redirected public request consumes the second admission normally.
        sourceRequestGate.recordSyntheticPublicAdmission(
          returnsGatedPublicRedirect ? 1 : 2,
        );
      } else if (
        sourcePath.startsWith("/local--code/") &&
        !returnsGatedPublicRedirect
      ) {
        // Authored/generated local-code URLs are direct wdfiles requests on
        // Wikidot. A successful local mirror therefore records one synthetic
        // public admission without delaying the local request. A fallback
        // public redirect is already gated normally as an actual network request.
        sourceRequestGate.recordSyntheticPublicAdmission();
      }
    }
    if (
      sourceRequestGate !== null &&
      responseCache?.evidenceReplay === true &&
      route.request().method?.() === "GET" &&
      returnsGatedPublicRedirect
    ) {
      const resolved = await resolveEvidenceReplaySubresourceRedirect({
        route,
        gate: sourceRequestGate,
        responseCache,
        resourceType: route.request().resourceType?.() ?? "other",
        sourceUrl: requestUrl.href,
        entry: {
          status: responseStatus,
          headers: responseHeaders,
          body: Buffer.alloc(0),
        },
        fetchRetainedEntry: (targetUrl) => fetchBrowserEvidenceResponse({
          route,
          gate: sourceRequestGate,
          responseCache,
          request: route.request(),
          targetUrl,
        }),
      });
      if (resolved?.entry) {
        await route.fulfill(
          evidenceReplaySubresourceFulfillment(
            route.request().resourceType?.() ?? "other",
            resolved,
          ),
        );
        return;
      }
      if (resolved?.response) {
        await route.fulfill({ response: resolved.response });
        return;
      }
    }
    await route.fulfill(
      replayedResponse
        ? {status: response.status, headers: response.headers, body: response.body}
        : {response},
    );
  };
  // Framerail can emit either Wikidot's canonical no-port file authority or
  // the candidate's already-localized sealed-port authority. Both represent
  // the same source-owned file request and must pass through the timing shim;
  // otherwise already-localized assets bypass the source request gate and can
  // complete before Wikidot's DOMContentLoaded-immediate observation.
  await context.route(`${canonicalFilesOrigin}/**`, fileRouteHandler);
  if (files.origin !== canonicalFilesOrigin) {
    await context.route(`${files.origin}/**`, fileRouteHandler);
  }
  if (responseCache !== null) {
    await context.route(`${page.origin}/local--files/**`, fileRouteHandler);
    await context.route(`${page.origin}/local--code/**`, fileRouteHandler);
  }
  return true;
}

export function candidateLocalOriginSets(candidate) {
  const endpointOrigins =
    candidate?.candidate?.endpoint?.allowed_origin_set ?? [];
  const siteOrigins = candidate?.candidate?.site_origins;
  const fileRouteOriginSets =
    siteOrigins && Object.keys(siteOrigins).length > 0
      ? Object.values(siteOrigins).map(({ page, files }) => [page, files])
      : endpointOrigins.length > 0
        ? [endpointOrigins]
        : [];
  const localOrigins = [
    ...new Set([...endpointOrigins, ...fileRouteOriginSets.flat()]),
  ].sort();
  return { localOrigins, fileRouteOriginSets };
}

export function parityBrowserThrottleConfig({
  args,
  runId,
  lock,
  policy,
  localOrigins,
  candidate,
  credentialPolicy = "none",
  publicOrigins = [],
}) {
  const executionMode = parityBrowserExecutionMode(args.mode);
  const caseSetPublicOrigins = requireExactHttpsOrigins(
    publicOrigins,
    "browser public origins",
  );
  let credentials = "none";
  if (credentialPolicy !== "none") {
    const value = requirePlainObject(
      credentialPolicy,
      "browser credential policy",
    );
    if (
      value.mode !== "private-actor-storage-states" ||
      !Number.isSafeInteger(value.storage_state_count) ||
      value.storage_state_count < 1 ||
      JSON.stringify(Object.keys(value).sort()) !==
        JSON.stringify([
          "mode",
          "private_input_identity_sha256",
          "storage_state_count",
        ])
    ) {
      throw new Error(
        "browser credential policy must bind a private actor storage-state count",
      );
    }
    credentials = {
      mode: value.mode,
      storage_state_count: value.storage_state_count,
      private_input_identity_sha256: requireSha256(
        value.private_input_identity_sha256,
        "browser credential policy private input identity SHA-256",
      ),
    };
  }
  return {
    schema: THROTTLE_CONFIG_SCHEMA,
    status: "sealed_before_browser_request",
    run_id: runId,
    mode: args.mode,
    execution_mode: executionMode,
    interval_ms: parityBrowserRequestIntervalMs(args.mode),
    browser_capture_lock: { path: lock.path, owner: lock.owner },
    live_completion_policy: {
      sha256: policy.sha256,
      policy_version: policy.value.policy_version,
    },
    local_context_exempt_origins: localOrigins,
    candidate_endpoint: candidate ?? null,
    ...(caseSetPublicOrigins.length === 0
      ? {
          public_request_policy:
            "Wikidot-family requests and non-Wikidot stylesheets, fonts, and images are admitted by the shared persistent gate; scripts and fetches from other public origins are aborted before admission",
          public_origin_policy:
            "HTTP(S) Wikidot page/resource hosts (wikidot.com and its subdomains, wdfiles.com resources, /v-- static assets on a CloudFront host, and exact HTTPS GET interwiki.scpwiki.com styleFrame/interwikiFrame documents plus interwiki/resizeIframe scripts) are gated; non-Wikidot stylesheet, font, and image dependencies are gated by resource type; other public hosts are aborted before admission",
        }
      : {
          case_set_public_origins: caseSetPublicOrigins,
          public_request_policy:
            "Wikidot-family requests, exact case-set GET origins, and non-Wikidot stylesheets, fonts, and images are admitted by the shared persistent gate; scripts and fetches from other public origins are aborted before admission",
          public_origin_policy:
            "HTTP(S) Wikidot page/resource hosts (wikidot.com and its subdomains, wdfiles.com resources, /v-- static assets on a CloudFront host, and exact HTTPS GET interwiki.scpwiki.com styleFrame/interwikiFrame documents plus interwiki/resizeIframe scripts) and exact case-set HTTPS GET origins are gated; non-Wikidot stylesheet, font, and image dependencies are gated by resource type; other public hosts are aborted before admission",
        }),
    service_workers: "block",
    web_sockets: "blocked_without_network_connection",
    credentials,
  };
}

export function isParityBrowserPublicOrigin(
  value,
  resourceType,
  method,
  publicOrigins = [],
  initiatorUrl = null,
) {
  const url = value instanceof URL ? value : new URL(value);
  return (
    isWikidotCapturePublicOrigin(url, resourceType, method, initiatorUrl) ||
    (method === "GET" && publicOrigins.includes(url.origin))
  );
}

export async function createParityBrowserControls({
  args,
  outputDir,
  policy,
  candidate,
  credentialPolicy = "none",
  publicOrigins = [],
  resume = false,
  responseCacheOptions = null,
}) {
  const runId = randomUUID();
  const executionMode = parityBrowserExecutionMode(args.mode);
  // Live-reference capture shares one host-global admission state. A caller
  // must not be able to select a second lock/state pair from the public CLI.
  const lock = await acquireBrowserCaptureLock({ runId });
  let gate = null;
  let proxy = null;
  let responseCache = null;
  try {
    gate = await createPersistentBrowserRequestGate({
      statePath: lock.statePath,
      intervalMs: parityBrowserRequestIntervalMs(args.mode),
    });
    if (responseCacheOptions !== null) {
      responseCache = createBrowserResponseCache(responseCacheOptions);
      await responseCache.load();
    }
    const { localOrigins, fileRouteOriginSets } =
      candidateLocalOriginSets(candidate);
    const caseSetPublicOrigins = requireExactHttpsOrigins(
      publicOrigins,
      "browser public origins",
    );
    const configPath = path.join(outputDir, "throttle-config-receipt.json");
    let configRunId = runId;
    let configLock = lock;
    if (resume) {
      const existing = await readJsonObject(configPath, "existing throttle config");
      configRunId = requireNonEmptyString(existing.run_id, "existing throttle config run_id");
      configLock = requirePlainObject(existing.browser_capture_lock, "existing throttle config browser_capture_lock");
    }
    const configSeal = await sealJsonNoReplace(
      configPath,
      parityBrowserThrottleConfig({
        args,
        runId: configRunId,
        lock: configLock,
        policy,
        localOrigins,
        candidate: candidate?.candidate.endpoint ?? null,
        credentialPolicy,
        publicOrigins: caseSetPublicOrigins,
      }),
    );
    proxy = await startCaptureEgressProxy({
      allowedLocalOrigins: localOrigins,
      requestTimeoutMs: args.timeoutMs,
      ...(candidate
        ? {
            lookup: localConnectLookup(
              candidate.candidate.endpoint.local_connect_address,
              localOrigins,
            ),
          }
        : {}),
    });
    return {
      gate,
      proxy,
      lock,
      runId,
      configPath,
      configSha256: configSeal.sha256,
      responseCache,
      localOrigins,
      fileRouteOriginSets,
      allowDefaultHttpsFileRoute: args.mode === "standing",
      publicOrigins: caseSetPublicOrigins,
      async close() {
        let failure = null;
        await proxy?.close().catch((error) => {
          failure ??= error;
        });
        await responseCache?.flush().catch((error) => {
          failure ??= error;
        });
        await gate.flush().catch((error) => {
          failure ??= error;
        });
        const finalGateSnapshot = failure
          ? null
          : {
          ...gate.snapshot(),
          execution_mode: executionMode,
          config_sha256: configSeal.sha256,
          response_cache: responseCache?.snapshot() ?? null,
            };
        if (!failure) {
          await lock.confirmState().catch((error) => {
            failure ??= error;
          });
        }
        if (!failure) {
          await lock.release().catch((error) => {
            failure ??= error;
          });
        }
        if (failure) throw failure;
        return finalGateSnapshot;
      },
      setActiveFixture(fixtureId) {
        gate.setActiveFixture(fixtureId);
      },
    };
  } catch (error) {
    await proxy?.close().catch(() => undefined);
    await responseCache?.flush().catch(() => undefined);
    if (gate) {
      const flushed = await gate
        .flush()
        .then(() => true)
        .catch(() => false);
      if (flushed) {
        await lock.confirmState().catch(() => undefined);
        await lock.release().catch(() => undefined);
      }
    }
    throw error;
  }
}

export async function launchParityBrowser({
  browserRoot,
  browserExecutable,
  controls,
  local,
  storageState = null,
  viewport,
  responseCache = null,
}) {
  const { chromium } = requirePlaywright(browserRoot);
  const executable = await resolveBrowserExecutable(
    chromium,
    browserExecutable,
  );
  const browser = await chromium.launch(parityBrowserLaunchOptions(executable));
  let context = null;
  try {
    context = await browser.newContext({
      ignoreHTTPSErrors: local,
      viewport,
      deviceScaleFactor: 1,
      colorScheme: "light",
      reducedMotion: "reduce",
      serviceWorkers: "block",
      proxy: { server: controls.proxy.url, bypass: "<-loopback>" },
      ...(storageState === null ? {} : { storageState }),
    });
    const requestGateAttribution = await installBrowserRequestGate(context, {
      gate: controls.gate,
      exemptOrigins: local ? controls.localOrigins : [],
      responseCache,
      cacheOnly: local,
      publicOriginPredicate: (url, resourceType, method, initiatorUrl) =>
        isParityBrowserPublicOrigin(
          url,
          resourceType,
          method,
          controls.publicOrigins,
          initiatorUrl,
        ),
    });
    if (local) {
      for (const originSet of controls.fileRouteOriginSets) {
        await installCandidateFilePortRoute(context, originSet, {
          sourceRequestGate: controls.gate,
          responseCache,
          allowDefaultHttpsPort: controls.allowDefaultHttpsFileRoute === true,
        });
      }
    }
    return {
      browser,
      context,
      requestGateAttribution,
      environment: {
        engine: "chromium",
        version: await browser.version(),
        executable_sha256: await sha256File(executable),
      },
      async close() {
        await closeParityBrowserResources(context, browser);
      },
    };
  } catch (error) {
    try {
      await closeParityBrowserResources(context, browser);
    } catch (cleanupError) {
      throw new AggregateError(
        [error, cleanupError],
        "browser initialization and cleanup both failed",
      );
    }
    throw error;
  }
}

async function boundedBrowserClose(close, timeoutMs, label) {
  let timer = null;
  try {
    return await Promise.race([
      Promise.resolve().then(close),
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error(`${label} close exceeded ${timeoutMs}ms`)),
          timeoutMs,
        );
      }),
    ]);
  } finally {
    if (timer !== null) clearTimeout(timer);
  }
}

export async function closeParityBrowserResources(
  context,
  browser,
  { timeoutMs = 10_000 } = {},
) {
  const failures = [];
  if (context !== null) {
    await boundedBrowserClose(
      () => context.close(),
      timeoutMs,
      "context",
    ).catch((error) => failures.push(error));
  }
  await boundedBrowserClose(
    () => browser.close(),
    timeoutMs,
    "browser",
  ).catch((error) => failures.push(error));
  if (failures.length > 0) {
    throw new AggregateError(
      failures,
      "parity browser resources failed to close",
    );
  }
}
