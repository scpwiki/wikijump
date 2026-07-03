import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../../../..');
const scriptPath = path.join(repoRoot, 'install/local/wikidot-verification/scripts/capture-corpus-files.mjs');

test('capture-corpus-files dry-run discovers absolute and page-relative attachments', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'capture-corpus-files-'));
  const pageDir = path.join(root, 'en', 'pages', 'scp-1234');
  fs.mkdirSync(pageDir, { recursive: true });
  fs.writeFileSync(
    path.join(pageDir, 'source.wikidot.txt'),
    [
      '[[image cover.jpg]]',
      '[[image alternate.jpg?rev=1]]',
      '[[include component:image-block',
      'name=detail.png|caption=Detail]]',
      '[[include component:image-block',
      'name=detail-v2.png#caption|caption=Detail]]',
      '[[module CSS]]',
      '.icon { background: url(icon.svg#cache); }',
      '[[/module]]',
      '[[image https://scp-wiki.wdfiles.com/local--files/scp-1234/remote.webp]]',
      '> **Filename:** credits-only.gif, cover.jpg',
    ].join('\n'),
  );

  const result = spawnSync(
    process.execPath,
    [scriptPath, '--corpus-root', root, '--branch', 'en', '--slug', 'scp-1234', '--dry-run'],
    { encoding: 'utf8' },
  );

  assert.equal(result.status, 0, result.stderr);
  const rows = result.stdout
    .split('\n')
    .filter((line) => line.startsWith('{"action"'))
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    rows.map((row) => row.filename).sort(),
    ['alternate.jpg', 'cover.jpg', 'credits-only.gif', 'detail-v2.png', 'detail.png', 'icon.svg', 'remote.webp'],
  );
  assert.equal(rows.find((row) => row.filename === 'remote.webp').original_url, 'https://scp-wiki.wdfiles.com/local--files/scp-1234/remote.webp');
});
