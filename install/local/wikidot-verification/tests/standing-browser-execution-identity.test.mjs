import assert from "node:assert/strict";
import test from "node:test";

import {
  STANDING_BROWSER_EXECUTION_MODULES,
  validateCandidateExecutionIdentity,
} from "../src/standing-browser-execution-identity.mjs";
import { sha256Value } from "../src/standing-browser-parity-util.mjs";

const hash = (character) => character.repeat(64);
const git = (character) => character.repeat(40);

function candidateIdentity() {
  return {
    candidate: {
      wikijump_commit: git("a"),
      wikijump_tree: git("b"),
      ftml_sha: git("c"),
    },
  };
}

function executionIdentity() {
  const modules = [...STANDING_BROWSER_EXECUTION_MODULES]
    .sort()
    .map((filePath) => ({ path: filePath, sha256: hash("d") }));
  return {
    schema: "wikijump.standing_browser_execution_identity.v1",
    source_clean: true,
    wikijump_commit: git("a"),
    wikijump_tree: git("b"),
    ftml_sha: git("c"),
    modules,
    module_manifest_sha256: sha256Value(modules),
  };
}

test("execution identity binds a clean exact source tree and every loaded parity module", () => {
  assert.equal(
    validateCandidateExecutionIdentity(executionIdentity(), candidateIdentity())
      .modules.length,
    STANDING_BROWSER_EXECUTION_MODULES.length,
  );
});

test("execution identity rejects a partial module manifest or substituted module hash", () => {
  const partial = executionIdentity();
  partial.modules.pop();
  partial.module_manifest_sha256 = sha256Value(partial.modules);
  assert.throws(
    () => validateCandidateExecutionIdentity(partial, candidateIdentity()),
    /incomplete module manifest/u,
  );

  const substituted = executionIdentity();
  substituted.modules[0].sha256 = hash("e");
  assert.throws(
    () => validateCandidateExecutionIdentity(substituted, candidateIdentity()),
    /module manifest hash is invalid/u,
  );
});
