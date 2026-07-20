import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import {
  AtomicPublicationAmbiguousError,
  publishBytesNoReplaceAt,
} from "./atomic-no-replace.mjs";
import { isPathInside } from "./corpus-file-reader.mjs";
import { openCorpusOutputDirectory } from "./corpus-output-writer.mjs";

export const XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES = Object.freeze({
  clusters: "mismatch-clusters.json",
  manifest: "verified-pilot-manifest.jsonl",
  rows: "local-comparison.jsonl",
  verdict: "xmlrpc-pilot-verdict.json",
});

async function assertPrivateComparisonOutputDirectory(directory) {
  const stat = await directory.stat({ bigint: true });
  if (!stat.isDirectory()) {
    throw new Error("comparison output directory must be a directory");
  }
  if (stat.uid !== BigInt(process.geteuid())) {
    throw new Error(
      "comparison output directory must be owned by the current user",
    );
  }
  if ((stat.mode & 0o777n) !== 0o700n) {
    throw new Error("comparison output directory must have mode 700");
  }
}

export async function openPrivateComparisonOutputDirectory({
  outputDir,
  pilotRoot,
} = {}) {
  if (typeof outputDir !== "string" || !path.isAbsolute(outputDir)) {
    throw new Error("outputDir must be an absolute path");
  }
  const root = await fs.realpath(pilotRoot);
  const output = path.resolve(outputDir);
  if (isPathInside(root, output)) {
    throw new Error("outputDir must be outside pilotRoot");
  }
  const directory = await openCorpusOutputDirectory(root, output);
  try {
    await assertPrivateComparisonOutputDirectory(directory);
    return directory;
  } catch (error) {
    await directory.close().catch(() => {});
    throw error;
  }
}

async function readOutputAt(directory, name, expectedBytes) {
  const handle = await fs.open(
    `/proc/self/fd/${directory.fd}/${name}`,
    fsConstants.O_RDONLY |
      (fsConstants.O_NONBLOCK ?? 0) |
      (fsConstants.O_NOFOLLOW ?? 0),
  );
  try {
    const before = await handle.stat({ bigint: true });
    if (
      !before.isFile() ||
      before.uid !== BigInt(process.geteuid()) ||
      (before.mode & 0o777n) !== 0o400n ||
      before.size !== BigInt(expectedBytes.byteLength)
    ) {
      throw new Error(
        `comparison output ${name} is not the expected private regular file`,
      );
    }
    const bytes = await handle.readFile();
    const after = await handle.stat({ bigint: true });
    if (
      bytes.byteLength !== Number(before.size) ||
      after.dev !== before.dev ||
      after.ino !== before.ino ||
      after.size !== before.size ||
      after.mtimeNs !== before.mtimeNs ||
      after.ctimeNs !== before.ctimeNs ||
      after.mode !== before.mode ||
      after.uid !== before.uid ||
      after.gid !== before.gid
    ) {
      throw new Error(`comparison output ${name} changed while being read`);
    }
    return bytes;
  } finally {
    await handle.close();
  }
}

async function publishExactAt(directory, name, bytes) {
  let disposition;
  try {
    disposition = await publishBytesNoReplaceAt(directory, name, bytes, {
      mode: 0o400,
    });
  } catch (error) {
    if (
      !(error instanceof AtomicPublicationAmbiguousError) ||
      !error.published
    ) {
      throw error;
    }
    if (!(await readOutputAt(directory, name, bytes)).equals(bytes)) {
      throw new Error(
        `ambiguous comparison output ${name} could not be verified`,
        {
          cause: error,
        },
      );
    }
    disposition = "ambiguous_verified";
  }
  if (
    disposition === "exists" &&
    !(await readOutputAt(directory, name, bytes)).equals(bytes)
  ) {
    throw new Error(`comparison output ${name} conflicts with existing bytes`);
  }
}

export async function publishXmlrpcPilotLocalComparisonOutputs(
  directory,
  { clusters, manifest, rows, verdict } = {},
) {
  await publishExactAt(
    directory,
    XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.rows,
    rows,
  );
  await publishExactAt(
    directory,
    XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.clusters,
    clusters,
  );
  await publishExactAt(
    directory,
    XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.manifest,
    manifest,
  );
  await publishExactAt(
    directory,
    XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.verdict,
    verdict,
  );
}
