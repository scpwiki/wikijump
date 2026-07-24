import crypto from 'node:crypto';
import fs from 'node:fs';
import { parentPort } from 'node:worker_threads';

export const CORPUS_SNAPSHOT_HASH_WORKER_URL = new URL(import.meta.url);

export function hashCorpusSnapshotPaths(paths) {
  return paths.map((filePath) => {
    const bytes = fs.readFileSync(filePath);
    return [filePath, {
      bytes: bytes.length,
      sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
    }];
  });
}

if (parentPort !== null) {
  parentPort.on('message', (paths) => {
    parentPort.postMessage(hashCorpusSnapshotPaths(paths));
  });
}
