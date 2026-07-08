import {execFileSync, spawn} from "node:child_process";
import {randomUUID} from "node:crypto";
import {createWriteStream} from "node:fs";
import {appendFile, mkdir} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";

const SAFE_FAMILY_RE = /^[A-Za-z0-9_.:-]+$/;
const STDERR_PREFIX_LIMIT = 2048;
const COMMON_SIGNAL_EXIT_CODES = new Map([
  ["SIGHUP", 129],
  ["SIGINT", 130],
  ["SIGQUIT", 131],
  ["SIGKILL", 137],
  ["SIGTERM", 143],
]);

function shellQuote(value) {
  const text = String(value);
  if (text.length === 0) {
    return "''";
  }
  return `'${text.replaceAll("'", "'\\''")}'`;
}

function createRunId() {
  return `${new Date().toISOString().replace(/[:.]/g, "-")}-${randomUUID().slice(0, 8)}`;
}

function validateOptions({family, label, command, args, timeoutMs}) {
  if (typeof family !== "string" || !SAFE_FAMILY_RE.test(family)) {
    throw new Error("family must match /^[A-Za-z0-9_.:-]+$/");
  }
  if (label !== null && typeof label !== "string") {
    throw new Error("label must be a string or null");
  }
  if (typeof command !== "string" || command.length === 0) {
    throw new Error("command must be a non-empty string");
  }
  if (!Array.isArray(args) || args.some((arg) => typeof arg !== "string")) {
    throw new Error("args must be an array of strings");
  }
  if (timeoutMs !== null && (!Number.isInteger(timeoutMs) || timeoutMs <= 0)) {
    throw new Error("timeoutMs must be a positive integer or null");
  }
}

function createBoundedUtf8Prefix(limit) {
  const decoder = new TextDecoder("utf-8", {fatal: false});
  let prefix = "";

  return {
    push(chunk) {
      if (prefix.length >= limit) {
        return;
      }
      prefix = `${prefix}${decoder.decode(chunk, {stream: true})}`.slice(0, limit);
    },
    finish() {
      if (prefix.length < limit) {
        prefix = `${prefix}${decoder.decode()}`.slice(0, limit);
      } else {
        decoder.decode();
      }
      return prefix;
    },
  };
}

function writeChunk(stream, chunk) {
  return new Promise((resolve, reject) => {
    function cleanup() {
      stream.off("drain", onDrain);
      stream.off("error", onError);
    }
    function onDrain() {
      cleanup();
      resolve();
    }
    function onError(error) {
      cleanup();
      reject(error);
    }

    stream.on("error", onError);
    if (stream.write(chunk)) {
      cleanup();
      resolve();
    } else {
      stream.on("drain", onDrain);
    }
  });
}

async function pumpOutput(readable, logStream, liveStream, {quiet, onChunk}) {
  let liveWritable = !quiet;
  for await (const chunk of readable) {
    onChunk(chunk);
    await writeChunk(logStream, chunk);
    if (liveWritable) {
      try {
        await writeChunk(liveStream, chunk);
      } catch {
        liveWritable = false;
      }
    }
  }
}

function finishStream(stream) {
  return new Promise((resolve, reject) => {
    stream.once("finish", resolve);
    stream.once("error", reject);
    stream.end();
  });
}

function sendSignalToChild(child, signal) {
  if (process.platform !== "win32" && Number.isInteger(child.pid)) {
    try {
      process.kill(-child.pid, signal);
      return;
    } catch {
      try {
        child.kill(signal);
      } catch {
        return;
      }
      return;
    }
  }
  try {
    child.kill(signal);
  } catch {
    return;
  }
}

function wrapperExitCode({exitCode, signal, timedOut, spawnError}) {
  if (timedOut) {
    return 124;
  }
  if (spawnError !== null) {
    return spawnError.code === "ENOENT" ? 127 : 1;
  }
  if (exitCode !== null) {
    return exitCode;
  }
  if (signal !== null) {
    return COMMON_SIGNAL_EXIT_CODES.get(signal) ?? 1;
  }
  return 1;
}

export function defaultLedgerPath() {
  return path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "..",
    "artifacts",
    "command-ledger",
    "ledger.jsonl",
  );
}

export function buildRerunCommand({cwd, command, args}) {
  return `cd ${shellQuote(cwd)} && ${[command, ...args].map(shellQuote).join(" ")}`;
}

export function readGitHead(cwd) {
  try {
    const head = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    return /^[0-9a-f]{40}$/i.test(head) ? head : null;
  } catch {
    return null;
  }
}

export async function runMeasuredCommand(options) {
  const family = options.family;
  const label = options.label ?? null;
  const command = options.command;
  const args = options.args ?? [];
  const cwd = path.resolve(options.cwd ?? process.cwd());
  const ledgerPath = path.resolve(options.ledgerPath ?? defaultLedgerPath());
  const timeoutMs = options.timeoutMs ?? null;
  const quiet = options.quiet === true;
  validateOptions({family, label, command, args, timeoutMs});

  const runId = createRunId();
  const runDirectory = path.join(path.dirname(ledgerPath), "runs", runId);
  const stdoutPath = path.join(runDirectory, "stdout.log");
  const stderrPath = path.join(runDirectory, "stderr.log");
  await mkdir(runDirectory, {recursive: true});

  const startTime = new Date().toISOString();
  const startNs = process.hrtime.bigint();
  const gitHead = readGitHead(cwd);
  const stderrPrefix = createBoundedUtf8Prefix(STDERR_PREFIX_LIMIT);
  let stdoutBytes = 0;
  let stderrBytes = 0;
  let timedOut = false;
  let spawnError = null;
  let childClosed = false;
  let timeoutTimer = null;
  let killTimer = null;

  const stdoutLog = createWriteStream(stdoutPath, {flags: "w"});
  const stderrLog = createWriteStream(stderrPath, {flags: "w"});
  const child = spawn(command, args, {
    cwd,
    stdio: ["inherit", "pipe", "pipe"],
    detached: process.platform !== "win32",
  });

  const forwardSigint = () => sendSignalToChild(child, "SIGINT");
  const forwardSigterm = () => sendSignalToChild(child, "SIGTERM");
  const ignoreLiveStreamError = () => {};
  process.on("SIGINT", forwardSigint);
  process.on("SIGTERM", forwardSigterm);
  process.stdout.on("error", ignoreLiveStreamError);
  process.stderr.on("error", ignoreLiveStreamError);

  child.once("error", (error) => {
    spawnError = error;
  });

  if (timeoutMs !== null) {
    timeoutTimer = setTimeout(() => {
      timedOut = true;
      sendSignalToChild(child, "SIGTERM");
      killTimer = setTimeout(() => {
        if (!childClosed) {
          sendSignalToChild(child, "SIGKILL");
        }
      }, 5000);
    }, timeoutMs);
  }

  const stdoutPump = pumpOutput(child.stdout, stdoutLog, process.stdout, {
    quiet,
    onChunk(chunk) {
      stdoutBytes += chunk.length;
    },
  });
  const stderrPump = pumpOutput(child.stderr, stderrLog, process.stderr, {
    quiet,
    onChunk(chunk) {
      stderrBytes += chunk.length;
      stderrPrefix.push(chunk);
    },
  });

  const closeResult = await new Promise((resolve) => {
    child.once("close", (code, signal) => {
      resolve({code, signal});
    });
  });
  childClosed = true;
  process.off("SIGINT", forwardSigint);
  process.off("SIGTERM", forwardSigterm);
  if (timeoutTimer !== null) {
    clearTimeout(timeoutTimer);
  }
  if (killTimer !== null) {
    clearTimeout(killTimer);
  }

  try {
    await Promise.all([stdoutPump, stderrPump]);
    await Promise.all([finishStream(stdoutLog), finishStream(stderrLog)]);
  } finally {
    process.stdout.off("error", ignoreLiveStreamError);
    process.stderr.off("error", ignoreLiveStreamError);
  }

  const endTime = new Date().toISOString();
  const elapsedMs = Number(process.hrtime.bigint() - startNs) / 1_000_000;
  const stderrExcerpt = stderrPrefix.finish();
  const exitCode = timedOut ? null : closeResult.code;
  const signal = closeResult.signal ?? null;
  const failed = timedOut || spawnError !== null || exitCode !== 0 || signal !== null;
  const firstErrorExcerpt = failed
    ? (stderrExcerpt || (spawnError !== null ? spawnError.message : null))?.slice(0, STDERR_PREFIX_LIMIT) ?? null
    : null;
  const record = {
    schemaVersion: 1,
    runId,
    family,
    label,
    command,
    args,
    cwd,
    gitHead,
    startTime,
    endTime,
    elapsedMs,
    exitCode,
    signal,
    timedOut,
    timeoutMs,
    cacheStatus: "uncached",
    artifactPaths: {
      ledger: ledgerPath,
      stdout: stdoutPath,
      stderr: stderrPath,
    },
    stdoutBytes,
    stderrBytes,
    firstErrorExcerpt,
    rerunCommand: buildRerunCommand({cwd, command, args}),
    envFingerprint: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
    },
  };

  await appendFile(ledgerPath, `${JSON.stringify(record)}\n`, "utf8");

  return {
    record,
    exitCode: wrapperExitCode({exitCode, signal, timedOut, spawnError}),
  };
}
