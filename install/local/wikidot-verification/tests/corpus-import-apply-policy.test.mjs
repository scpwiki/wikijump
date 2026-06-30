import assert from 'node:assert/strict';
import test from 'node:test';

import { canReuseExistingPageForDbImport } from '../src/corpus-import-apply-policy.mjs';

test('canReuseExistingPageForDbImport reuses matching pages under replace-existing', () => {
  assert.equal(canReuseExistingPageForDbImport({ adoptExisting: false, replaceExisting: false }), false);
  assert.equal(canReuseExistingPageForDbImport({ adoptExisting: true, replaceExisting: false }), true);
  assert.equal(canReuseExistingPageForDbImport({ adoptExisting: false, replaceExisting: true }), true);
  assert.equal(
    canReuseExistingPageForDbImport(
      { adoptExisting: false, replaceExisting: false },
      { replaceExistingRevision: true },
    ),
    true,
  );
});
