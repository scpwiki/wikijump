import crypto from "node:crypto";
import {createRequire} from "node:module";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";

import {openBrowser} from "./browser-session.mjs";
import {captureThemeTierBrowserEvidence, prepareThemeArtifactDirectory} from "./theme-browser-capture.mjs";
import {DeepwellThemePageAdapter} from "./theme-localization-deepwell-adapter.mjs";
import {ALLOWED_SITE_SLUG, readCurrentSiteDependencySource} from "./theme-localization-e2e.mjs";
import {executeThemeRunOwnedPages, recoverThemeExecution, themeExecutionFingerprint, validateRecoverableThemeExecutionPlan, validateThemeExecutionPlan} from "./theme-localization-execution.mjs";
import {WikidotThemePageAdapter} from "./theme-localization-wikidot-adapter.mjs";

export const THEME_RUN_RESULT_SCHEMA = "wikijump_local_lab.theme_run_result.v1";
export const GUARDED_THEME_WIKIJUMP_RPC_URL = "http://127.0.0.1:12747/jsonrpc";
const DEFAULT_BROWSER_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..", "framerail");
const DEFAULT_EXECUTION_LOCK = path.join(os.tmpdir(), `wikijump-theme-localization-${ALLOWED_SITE_SLUG}.lock`);

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function requiredEnv(env, name) {
  if (typeof env[name] !== "string" || !env[name]) throw new Error(`${name} is required`);
  return env[name];
}

function redact(message, secrets) {
  let result = String(message);
  for (const secret of secrets.filter((value) => typeof value === "string" && value.length >= 4)) result = result.replaceAll(secret, "[REDACTED]");
  return result;
}

async function readAcceptedSource(resource) {
  const stat = await fs.lstat(resource.source_path);
  if (!stat.isFile() || stat.isSymbolicLink()) throw new Error(`accepted source is not a regular file: ${resource.resource_id}`);
  return fs.readFile(resource.source_path, "utf8");
}

export async function validateStorageState(filePath) {
  if (!filePath) return null;
  const absolute = path.resolve(filePath);
  const stat = await fs.lstat(absolute);
  if (!stat.isFile() || stat.isSymbolicLink()) throw new Error("browser storage state must be a regular file");
  if ((stat.mode & 0o077) !== 0) throw new Error("browser storage state permissions must deny group and other access");
  return absolute;
}

function loadChromium(browserRoot = DEFAULT_BROWSER_ROOT) {
  const requireFromRoot = createRequire(path.join(path.resolve(browserRoot), "package.json"));
  try {
    return requireFromRoot("playwright").chromium;
  } catch (error) {
    try {
      return requireFromRoot("@playwright/test").chromium;
    } catch (fallbackError) {
      throw new Error(`Playwright is unavailable (${error.code ?? "load_failed"}; ${fallbackError.code ?? "load_failed"})`);
    }
  }
}

export function validateThemeCdpEndpoint(value) {
  const url = new URL(value);
  if (url.protocol !== "http:" || !new Set(["127.0.0.1", "localhost"]).has(url.hostname) || !url.port || Number(url.port) < 1 || url.pathname !== "/" || url.username || url.password || url.search || url.hash) {
    throw new Error("theme CDP endpoint must be an uncredentialed loopback HTTP origin with an explicit port");
  }
  return url.origin;
}

export function validateGuardedThemeRpcUrl(value) {
  if (value !== GUARDED_THEME_WIKIJUMP_RPC_URL) {
    throw new Error(`WIKIJUMP_THEME_RPC_URL must explicitly equal ${GUARDED_THEME_WIKIJUMP_RPC_URL}`);
  }
  return value;
}

export async function createLiveThemeDependencies({env = process.env, browserRoot, browserExecutable, cdpEndpoint, wikidotStorageState, wikijumpStorageState, ignoreHttpsErrors = false, needsBrowser = true, openBrowserImpl = openBrowser} = {}) {
  if (cdpEndpoint && browserExecutable) throw new Error("CDP endpoint cannot be combined with a browser executable");
  const validatedCdpEndpoint = cdpEndpoint ? validateThemeCdpEndpoint(cdpEndpoint) : null;
  const rpcUrl = validateGuardedThemeRpcUrl(env.WIKIJUMP_THEME_RPC_URL);
  const secrets = [requiredEnv(env, "WIKIDOT_USERNAME"), requiredEnv(env, "WIKIDOT_PASSWORD"), requiredEnv(env, "WIKIJUMP_THEME_ADMIN_EMAIL"), requiredEnv(env, "WIKIJUMP_THEME_ADMIN_PASSWORD")];
  const actorUserId = env.WIKIJUMP_THEME_ACTOR_USER_ID === undefined ? -1 : Number(env.WIKIJUMP_THEME_ACTOR_USER_ID);
  if (!Number.isSafeInteger(actorUserId)) throw new Error("WIKIJUMP_THEME_ACTOR_USER_ID must be an integer");
  const storageStates = needsBrowser ? {wikidot: await validateStorageState(wikidotStorageState), wikijump: await validateStorageState(wikijumpStorageState)} : {};
  const wikidot = new WikidotThemePageAdapter({helperOptions: {env}});
  const wikijump = new DeepwellThemePageAdapter({rpcUrl, adminEmail: env.WIKIJUMP_THEME_ADMIN_EMAIL, adminPassword: env.WIKIJUMP_THEME_ADMIN_PASSWORD, actorUserId});
  let browserSession = null;
  try {
    await wikidot.connect();
    await wikijump.connect();
    const chromium = needsBrowser ? loadChromium(browserRoot) : null;
    if (needsBrowser) browserSession = await openBrowserImpl({chromium, cdpEndpoint: validatedCdpEndpoint, browserExecutable, ignoreHttpsErrors, createInitialContexts: false});
    return {
      adapters: {wikidot, wikijump}, secrets, storageStates, browserExecutable, cdpEndpoint: validatedCdpEndpoint, ignoreHttpsErrors, chromium, browserSession,
      async close() { await Promise.allSettled([browserSession?.close(), wikijump.close(), wikidot.close()]); },
    };
  } catch (error) {
    await Promise.allSettled([browserSession?.close(), wikijump.close(), wikidot.close()]);
    throw error;
  }
}

function installSignalBridge(signalSource) {
  const controller = new AbortController();
  let received = null;
  const listeners = new Map(["SIGINT", "SIGTERM"].map((name) => [name, () => {
    if (!received) {
      received = name;
      const error = new Error(`theme execution interrupted by ${name}`);
      error.signal = name;
      controller.abort(error);
    }
  }]));
  for (const [name, listener] of listeners) signalSource.once(name, listener);
  return {signal: controller.signal, received: () => received, close() { for (const [name, listener] of listeners) signalSource.off(name, listener); }};
}

function validateExecutablePlan(plan, {recovery = false} = {}) {
  (recovery ? validateRecoverableThemeExecutionPlan : validateThemeExecutionPlan)(plan);
  if (plan.mode !== "execute" || plan.safety?.execute_supported !== true) throw new Error("theme plan is not explicitly executable");
}

async function syncParent(filePath) {
  const handle = await fs.open(path.dirname(filePath), "r");
  try { await handle.sync(); } finally { await handle.close(); }
}

function processIsAlive(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === "EPERM";
  }
}

export async function acquireThemeExecutionLock({lockPath = DEFAULT_EXECUTION_LOCK, runId, fingerprint} = {}) {
  const absolute = path.resolve(lockPath);
  const parent = path.dirname(absolute);
  await fs.mkdir(parent, {recursive: true});
  const ownerPath = path.join(absolute, "owner.json");
  const create = async () => {
    await fs.mkdir(absolute, {mode: 0o700});
    const owner = {schema: "wikijump_local_lab.theme_execution_lock.v1", pid: process.pid, run_id: runId, fingerprint};
    const handle = await fs.open(ownerPath, "wx", 0o600);
    try {
      await handle.writeFile(`${JSON.stringify(owner)}\n`, "utf8");
      await handle.sync();
    } finally {
      await handle.close();
    }
    await syncParent(ownerPath);
  };
  try {
    await create();
  } catch (error) {
    if (error?.code !== "EEXIST") throw error;
    const stat = await fs.lstat(absolute);
    const ownerStat = await fs.lstat(ownerPath);
    if (!stat.isDirectory() || stat.isSymbolicLink() || !ownerStat.isFile() || ownerStat.isSymbolicLink() || (ownerStat.mode & 0o077) !== 0) throw new Error("theme execution lock is malformed");
    let owner;
    try {
      owner = JSON.parse(await fs.readFile(ownerPath, "utf8"));
    } catch {
      throw new Error("theme execution lock owner is malformed");
    }
    if (owner?.schema !== "wikijump_local_lab.theme_execution_lock.v1" || !Number.isSafeInteger(owner.pid) || typeof owner.run_id !== "string" || typeof owner.fingerprint !== "string") throw new Error("theme execution lock owner is malformed");
    if (processIsAlive(owner.pid)) throw new Error(`theme execution site lock is held by run ${owner.run_id}`);
    await fs.unlink(ownerPath);
    await fs.rmdir(absolute);
    await create();
  }
  let released = false;
  return {
    path: absolute,
    async release() {
      if (released) return;
      released = true;
      await fs.unlink(ownerPath);
      await fs.rmdir(absolute);
      await syncParent(absolute);
    },
  };
}

export async function writeExecutableThemePlan(filePath, plan) {
  validateExecutablePlan(plan);
  const absolute = path.resolve(filePath);
  await fs.mkdir(path.dirname(absolute), {recursive: true});
  const handle = await fs.open(absolute, "wx", 0o600);
  try {
    await handle.writeFile(`${JSON.stringify(plan, null, 2)}\n`, "utf8");
    await handle.sync();
  } finally {
    await handle.close();
  }
  await syncParent(absolute);
  return absolute;
}

async function reserveResult(filePath) {
  await fs.mkdir(path.dirname(filePath), {recursive: true});
  const handle = await fs.open(filePath, "wx", 0o600);
  return {
    async write(value) {
      await handle.writeFile(`${JSON.stringify(value, null, 2)}\n`, "utf8");
      await handle.sync();
      await handle.close();
    },
    async close() { await handle.close().catch(() => {}); },
  };
}

function captureSummary(captures) {
  return captures.map((capture) => ({tier_id: capture.tier_id, status: capture.status, targets: capture.targets.map((target) => ({id: target.id, status: target.verdict.status, failed_viewports: target.verdict.failed_viewports}))}));
}

export async function runGuardedThemeAction({mode, plan, ledgerPath, resultPath, artifactDir, executionLockPath, signalSource = process, dependencyFactory = createLiveThemeDependencies, dependencyOptions = {}, captureTierImpl = captureThemeTierBrowserEvidence}) {
  if (!new Set(["execute", "recover"]).has(mode)) throw new Error("theme action must be execute or recover");
  validateExecutablePlan(plan, {recovery: mode === "recover"});
  if (!ledgerPath || !resultPath || (mode === "execute" && !artifactDir)) throw new Error("ledger, result, and execute artifact paths are required");
  if (mode === "execute") artifactDir = await prepareThemeArtifactDirectory(artifactDir);
  const resultFile = await reserveResult(path.resolve(resultPath));
  const bridge = installSignalBridge(signalSource);
  const captures = [];
  let dependencies = null;
  let executionLock = null;
  let operation = null;
  let failure = null;
  try {
    executionLock = await acquireThemeExecutionLock({lockPath: executionLockPath, runId: plan.run.id, fingerprint: themeExecutionFingerprint(plan, {allowLegacy: mode === "recover"})});
    dependencies = await dependencyFactory({...dependencyOptions, needsBrowser: mode === "execute"});
    if (mode === "recover") {
      operation = await recoverThemeExecution({ledgerPath, plan, adapters: dependencies.adapters});
      if (bridge.signal.aborted) throw bridge.signal.reason;
    } else {
      operation = await executeThemeRunOwnedPages({
        plan, ledgerPath, adapters: dependencies.adapters, signal: bridge.signal,
        materialize: async (resource) => ({source: resource.kind === "component_dependency" ? await readCurrentSiteDependencySource(resource) : await readAcceptedSource(resource)}),
        capture: async (tier, resources) => {
          const themeResource = resources.find((resource) => resource.kind === "theme_page");
          if (!themeResource) throw new Error(`theme page resource is missing before capture: ${tier.id}`);
          const source = await readAcceptedSource(themeResource);
          if (sha256(source) !== themeResource.source_sha256) throw new Error(`accepted source changed before capture: ${tier.id}`);
          const capture = await captureTierImpl({tier, outputDir: artifactDir, source, chromium: dependencies.chromium, browserExecutable: dependencies.browserExecutable, cdpEndpoint: dependencies.cdpEndpoint, browserSession: dependencies.browserSession, ignoreHttpsErrors: dependencies.ignoreHttpsErrors, storageStates: dependencies.storageStates});
          captures.push(capture);
          if (capture.status !== "pass") throw new Error(`strict browser verdict failed: ${tier.id}`);
        },
      });
    }
  } catch (error) {
    failure = error;
  } finally {
    bridge.close();
    try { await dependencies?.close?.(); } catch (error) { failure ??= error; }
    try { await executionLock?.release(); } catch (error) { failure ??= error; }
  }
  const secrets = dependencies?.secrets ?? [];
  const aggregate = {schema: THEME_RUN_RESULT_SCHEMA, status: failure ? "fail" : "pass", mode, run_id: plan.run.id, plan_fingerprint: themeExecutionFingerprint(plan, {allowLegacy: mode === "recover"}), ledger_path: path.resolve(ledgerPath), signal: bridge.received(), captures: captureSummary(captures), operation, error: failure ? redact(failure.message ?? failure, secrets) : null};
  try {
    await resultFile.write(aggregate);
  } catch (error) {
    await resultFile.close();
    if (failure) throw new AggregateError([failure, error], "theme action failed and its aggregate result could not be persisted");
    throw error;
  }
  if (failure) {
    const safe = new Error(aggregate.error);
    safe.signal = failure.signal;
    throw safe;
  }
  return aggregate;
}
