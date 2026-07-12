import crypto from 'node:crypto';
import fs from 'node:fs';
import { parentPort } from 'node:worker_threads';

parentPort.on('message', (paths) => {
  const results = paths.map((filePath) => {
    const bytes = fs.readFileSync(filePath);
    return [filePath, {
      bytes: bytes.length,
      sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
    }];
  });
  parentPort.postMessage(results);
});
