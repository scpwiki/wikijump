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
  installBrowserRequestGate,
} from "./browser-request-gate.mjs";
import { startCaptureEgressProxy } from "./capture-egress-proxy.mjs";
import {
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

function throttleConfig({
  args,
  runId,
  lock,
  policy,
  localOrigins,
  candidate,
}) {
  return {
    schema: THROTTLE_CONFIG_SCHEMA,
    status: "sealed_before_browser_request",
    run_id: runId,
    mode: args.mode,
    interval_ms: DEFAULT_REQUEST_INTERVAL_MS,
    browser_capture_lock: { path: lock.path, owner: lock.owner },
    live_completion_policy: {
      sha256: policy.sha256,
      policy_version: policy.value.policy_version,
    },
    local_context_exempt_origins: localOrigins,
    candidate_endpoint: candidate ?? null,
    public_request_policy:
      "every HTTP(S) request except exact candidate-local origins is admitted by the shared persistent gate",
    service_workers: "block",
    web_sockets: "blocked_without_network_connection",
    credentials: "none",
  };
}

export async function createParityBrowserControls({
  args,
  outputDir,
  policy,
  candidate,
}) {
  const runId = randomUUID();
  // Live-reference capture shares one host-global admission state. A caller
  // must not be able to select a second lock/state pair from the public CLI.
  const lock = await acquireBrowserCaptureLock({ runId });
  let gate = null;
  let proxy = null;
  try {
    gate = await createPersistentBrowserRequestGate({
      statePath: lock.statePath,
      intervalMs: DEFAULT_REQUEST_INTERVAL_MS,
    });
    const localOrigins = candidate?.candidate.endpoint.allowed_origin_set ?? [];
    const configPath = path.join(outputDir, "throttle-config-receipt.json");
    const configSeal = await sealJsonNoReplace(
      configPath,
      throttleConfig({
        args,
        runId,
        lock,
        policy,
        localOrigins,
        candidate: candidate?.candidate.endpoint ?? null,
      }),
    );
    proxy = await startCaptureEgressProxy({
      allowedLocalOrigins: localOrigins,
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
      localOrigins,
      async close() {
        let failure = null;
        await proxy?.close().catch((error) => {
          failure ??= error;
        });
        await gate.flush().catch((error) => {
          failure ??= error;
        });
        const finalGateSnapshot = failure
          ? null
          : { ...gate.snapshot(), config_sha256: configSeal.sha256 };
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
    };
  } catch (error) {
    await proxy?.close().catch(() => undefined);
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
  viewport,
}) {
  const { chromium } = requirePlaywright(browserRoot);
  const executable = await resolveBrowserExecutable(
    chromium,
    browserExecutable,
  );
  const browser = await chromium.launch({ executablePath: executable });
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
    });
    await installBrowserRequestGate(context, {
      gate: controls.gate,
      exemptOrigins: local ? controls.localOrigins : [],
    });
    return {
      browser,
      context,
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
      throw new AggregateError([error, cleanupError], "browser initialization and cleanup both failed");
    }
    throw error;
  }
}

export async function closeParityBrowserResources(context, browser) {
  const failures = [];
  if (context !== null) {
    await context.close().catch((error) => failures.push(error));
  }
  await browser.close().catch((error) => failures.push(error));
  if (failures.length > 0) {
    throw new AggregateError(failures, "parity browser resources failed to close");
  }
}
