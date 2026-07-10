import {execFileSync, spawn} from "node:child_process";
import {randomUUID} from "node:crypto";
import {createWriteStream} from "node:fs";
import {appendFile, mkdir} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";

const SAFE_FAMILY_RE = /^[A-Za-z0-9_.:-]+$/;
const STDERR_PREFIX_LIMIT = 2048;
const REDACTION = "[REDACTED]";
const SENSITIVE_VALUE_OPTIONS = new Set([
  "--session-token",
  "--db-url",
  "--attachment-s3-secret-access-key",
]);
const SENSITIVE_NAME_RE = /(?:password|passwd|pwd|secret|token|credential|access[_-]?key|db[_-]?url)/i;
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

function isSensitiveOptionName(option) {
  return SENSITIVE_VALUE_OPTIONS.has(option) || SENSITIVE_NAME_RE.test(option.replace(/^--?/, ""));
}

function redactConnectionUrl(value) {
  return String(value).replace(/([a-z][a-z0-9+.-]*:\/\/[^\s:/?#]+:)([^@\s/?#]+)(@)/gi, `$1${REDACTION}$3`);
}

function redactTextPatterns(value) {
  return redactConnectionUrl(String(value))
    .replace(/((?:password|passwd|pwd|secret|token|credential|access[_-]?key|db[_-]?url)\s*[:=]\s*)([^\s'"`]+)/gi, `$1${REDACTION}`);
}

function redactArgs(args) {
  const redacted = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    const equalsIndex = arg.indexOf("=");
    if (arg.startsWith("--") && equalsIndex > 0) {
      const option = arg.slice(0, equalsIndex);
      redacted.push(isSensitiveOptionName(option) ? `${option}=${REDACTION}` : redactTextPatterns(arg));
      continue;
    }
    if (arg.startsWith("--") && isSensitiveOptionName(arg) && index + 1 < args.length) {
      redacted.push(arg, REDACTION);
      index += 1;
      continue;
    }
    redacted.push(redactTextPatterns(arg));
  }
  return redacted;
}

function sensitiveOutputSecrets(args) {
  const secrets = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    const equalsIndex = arg.indexOf("=");
    if (arg.startsWith("--") && equalsIndex > 0) {
      const option = arg.slice(0, equalsIndex);
      const value = arg.slice(equalsIndex + 1);
      if (isSensitiveOptionName(option)) {
        secrets.push(value);
        const urlPassword = /[a-z][a-z0-9+.-]*:\/\/[^\s:/?#]+:([^@\s/?#]+)@/i.exec(value)?.[1];
        if (urlPassword !== undefined) {
          secrets.push(urlPassword);
        }
      }
    } else if (arg.startsWith("--") && isSensitiveOptionName(arg) && index + 1 < args.length) {
      secrets.push(args[index + 1]);
      const urlPassword = /[a-z][a-z0-9+.-]*:\/\/[^\s:/?#]+:([^@\s/?#]+)@/i.exec(args[index + 1])?.[1];
      if (urlPassword !== undefined) {
        secrets.push(urlPassword);
      }
      index += 1;
    }

    if (redactTextPatterns(arg) !== arg) {
      secrets.push(arg);
    }
  }
  return [...new Set(secrets.filter((secret) => secret.length >= 4))];
}

function exactSecretIntervals(text, secrets) {
  const matches = [];
  for (const secret of secrets) {
    let start = text.indexOf(secret);
    while (start !== -1) {
      matches.push({start, end: start + secret.length});
      start = text.indexOf(secret, start + 1);
    }
  }
  if (matches.length === 0) {
    return matches;
  }

  matches.sort((left, right) => left.start - right.start || right.end - left.end);
  const merged = [];
  for (const match of matches) {
    const previous = merged.at(-1);
    if (previous !== undefined && match.start <= previous.end) {
      previous.end = Math.max(previous.end, match.end);
    } else {
      merged.push({...match});
    }
  }
  return merged;
}

function safeUtf16PrefixLength(text, requestedLength) {
  if (requestedLength <= 0 || requestedLength >= text.length) {
    return requestedLength;
  }
  const before = text.charCodeAt(requestedLength - 1);
  const after = text.charCodeAt(requestedLength);
  const splitsSurrogatePair = before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff;
  return splitsSurrogatePair ? requestedLength - 1 : requestedLength;
}

export function createOutputRedactor(args) {
  const decoder = new TextDecoder("utf-8", {fatal: false});
  const encoder = new TextEncoder();
  const secrets = sensitiveOutputSecrets(args);
  const maxSecretLength = Math.max(0, ...secrets.map((secret) => secret.length));
  const tailLength = Math.max(0, maxSecretLength - 1);
  let pending = "";
  let coveredPrefixLength = 0;

  function redactPrefix(emitLength) {
    const intervals = exactSecretIntervals(pending, secrets)
      .map((interval) => ({...interval, markerEmitted: false}));
    if (coveredPrefixLength > 0) {
      intervals.push({start: 0, end: coveredPrefixLength, markerEmitted: true});
    }
    intervals.sort((left, right) => left.start - right.start || right.end - left.end);

    const merged = [];
    for (const interval of intervals) {
      const previous = merged.at(-1);
      if (previous !== undefined && interval.start <= previous.end) {
        previous.end = Math.max(previous.end, interval.end);
        previous.markerEmitted ||= interval.markerEmitted;
      } else {
        merged.push({...interval});
      }
    }

    let cursor = 0;
    let output = "";
    let nextCoveredPrefixLength = 0;
    for (const interval of merged) {
      if (interval.start >= emitLength) {
        break;
      }
      output += pending.slice(cursor, interval.start);
      if (!interval.markerEmitted) {
        output += REDACTION;
      }
      cursor = Math.min(interval.end, emitLength);
      if (interval.end > emitLength) {
        nextCoveredPrefixLength = interval.end - emitLength;
      }
    }
    output += pending.slice(cursor, emitLength);
    coveredPrefixLength = nextCoveredPrefixLength;
    return redactTextPatterns(output);
  }

  return {
    push(chunk) {
      pending += decoder.decode(chunk, {stream: true});
      if (pending.length <= tailLength) {
        return new Uint8Array();
      }
      const emitLength = safeUtf16PrefixLength(pending, pending.length - tailLength);
      if (emitLength === 0) {
        return new Uint8Array();
      }
      const output = redactPrefix(emitLength);
      pending = pending.slice(emitLength);
      return encoder.encode(output);
    },
    finish() {
      pending += decoder.decode();
      const output = encoder.encode(redactPrefix(pending.length));
      pending = "";
      coveredPrefixLength = 0;
      return output;
    },
  };
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
    if (stream.destroyed || stream.writable === false) {
      reject(new Error("stream is no longer writable"));
      return;
    }
    function cleanup() {
      stream.off("drain", onDrain);
      stream.off("error", onError);
      stream.off("close", onClose);
    }
    function onDrain() {
      cleanup();
      resolve();
    }
    function onError(error) {
      cleanup();
      reject(error);
    }
    function onClose() {
      cleanup();
      reject(new Error("stream closed before write completed"));
    }

    stream.on("error", onError);
    stream.on("close", onClose);
    if (stream.write(chunk)) {
      cleanup();
      resolve();
    } else {
      stream.on("drain", onDrain);
    }
  });
}

async function pumpOutput(readable, logStream, liveStream, {quiet, onChunk, redactor}) {
  let liveWritable = !quiet;
  const stopLiveWrites = () => {
    liveWritable = false;
  };
  liveStream.once("error", stopLiveWrites);
  liveStream.once("close", stopLiveWrites);

  async function writeRedactedChunk(chunk) {
    if (chunk.length === 0) {
      return;
    }
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

  for await (const chunk of readable) {
    await writeRedactedChunk(redactor.push(chunk));
  }
  await writeRedactedChunk(redactor.finish());
  liveStream.off("error", stopLiveWrites);
  liveStream.off("close", stopLiveWrites);
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
  await mkdir(runDirectory, {recursive: true, mode: 0o700});

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

  const redactedArgs = redactArgs(args);
  const stdoutLog = createWriteStream(stdoutPath, {flags: "w", mode: 0o600});
  const stderrLog = createWriteStream(stderrPath, {flags: "w", mode: 0o600});
  const child = spawn(command, args, {
    cwd,
    stdio: ["inherit", "pipe", "pipe"],
    detached: process.platform !== "win32",
  });

  const forwardSigint = () => sendSignalToChild(child, "SIGINT");
  const forwardSigterm = () => sendSignalToChild(child, "SIGTERM");
  const ignoreLiveStreamError = (error) => {
    if (error?.code === "EPIPE") {
      process.stdout.destroy();
      process.stderr.destroy();
    }
  };
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
    redactor: createOutputRedactor(args),
  });
  const stderrPump = pumpOutput(child.stderr, stderrLog, process.stderr, {
    quiet,
    onChunk(chunk) {
      stderrBytes += chunk.length;
      stderrPrefix.push(chunk);
    },
    redactor: createOutputRedactor(args),
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
    args: redactedArgs,
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
    rerunCommand: buildRerunCommand({cwd, command, args: redactedArgs}),
    envFingerprint: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
    },
  };

  await appendFile(ledgerPath, `${JSON.stringify(record)}\n`, {encoding: "utf8", mode: 0o600});

  return {
    record,
    exitCode: wrapperExitCode({exitCode, signal, timedOut, spawnError}),
  };
}
