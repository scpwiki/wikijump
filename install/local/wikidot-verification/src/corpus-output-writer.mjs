import crypto from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import { assertDescriptorTraversalSupport, isPathInside } from "./corpus-file-reader.mjs";

const DIRECTORY_FLAGS = fsConstants.O_RDONLY | fsConstants.O_DIRECTORY | fsConstants.O_NOFOLLOW;

function outputSymlinkError(outputDir) {
  return new Error(`--output-dir must not contain symbolic links: ${outputDir}`);
}

async function openOutputComponent(directoryHandle, component, outputDir) {
  const componentPath = `/proc/self/fd/${directoryHandle.fd}/${component}`;
  try {
    return await fs.open(componentPath, DIRECTORY_FLAGS);
  } catch (error) {
    if (error.code !== "ENOENT") {
      if (["ELOOP", "EMLINK", "ENOTDIR"].includes(error.code)) {
        throw outputSymlinkError(outputDir);
      }
      throw error;
    }
  }

  try {
    await fs.mkdir(componentPath, { mode: 0o700 });
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
  }

  try {
    return await fs.open(componentPath, DIRECTORY_FLAGS);
  } catch (error) {
    if (["ELOOP", "EMLINK", "ENOTDIR"].includes(error.code)) {
      throw outputSymlinkError(outputDir);
    }
    throw error;
  }
}

export async function openCorpusOutputDirectory(corpusRoot, outputDir) {
  await assertDescriptorTraversalSupport();
  const realCorpusRoot = await fs.realpath(corpusRoot);
  const parsed = path.parse(outputDir);
  let directoryHandle = await fs.open(parsed.root, DIRECTORY_FLAGS);

  try {
    for (const component of outputDir.slice(parsed.root.length).split(path.sep).filter(Boolean)) {
      const nextDirectoryHandle = await openOutputComponent(directoryHandle, component, outputDir);
      await directoryHandle.close();
      directoryHandle = nextDirectoryHandle;
    }

    const realOutputDir = await fs.realpath(`/proc/self/fd/${directoryHandle.fd}`);
    if (isPathInside(realCorpusRoot, realOutputDir)) {
      throw new Error("--output-dir must be outside --corpus to avoid self-inventory");
    }
    return directoryHandle;
  } catch (error) {
    await directoryHandle.close().catch(() => {});
    throw error;
  }
}

export async function writeCorpusOutputFile(directoryHandle, name, contents) {
  const temporaryName = `.corpus-discover-${crypto.randomUUID()}.tmp`;
  const temporaryPath = `/proc/self/fd/${directoryHandle.fd}/${temporaryName}`;
  const fileHandle = await fs.open(
    temporaryPath,
    fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL | fsConstants.O_NOFOLLOW,
    0o600,
  );
  try {
    await fileHandle.chmod(0o600);
    await fileHandle.writeFile(contents);
  } finally {
    await fileHandle.close();
  }

  try {
    await fs.rename(temporaryPath, `/proc/self/fd/${directoryHandle.fd}/${name}`);
  } catch (error) {
    await fs.unlink(temporaryPath).catch(() => {});
    throw error;
  }
}
