import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  LOCKFILES,
  buildFinalFrozenReceipt,
  parseArgs as parseFinalFrozenArgs,
} from "../scripts/emit-final-frozen-receipt.mjs";
import {parsePreflightOutput} from "../scripts/run-consolidated-validation.mjs";
import {
  CONSOLIDATED_VALIDATION_SCHEMA,
  validateConsolidatedValidation,
  verifyFinalFrozenReceipt,
} from "../src/final-frozen-receipt-contract.mjs";

const SOURCE = Object.freeze({
  wikijump_commit: "a".repeat(40),
  wikijump_tree: "b".repeat(40),
  ftml_sha: "c".repeat(40),
});

function validation(overrides = {}) {
  return {
    schema: CONSOLIDATED_VALIDATION_SCHEMA,
    status: "pass",
    started_at: "2026-10-15T00:00:00.000Z",
    completed_at: "2026-10-15T01:00:00.000Z",
    wikijump_commit: SOURCE.wikijump_commit,
    wikijump_tree: SOURCE.wikijump_tree,
    base_commit: "d".repeat(40),
    mode: "final",
    groups: {deepwell: true, verification: true},
    checks: [
      {name: "deepwell clippy", status: "ok"},
      {name: "deepwell full tests", status: "ok"},
    ],
    tool_versions: {node: "v24"},
    preflight: {exit: 0, log: {path: "/log", sha256: "e".repeat(64)}, source_sha256: {}},
    ...overrides,
  };
}

function imageProducer(source = SOURCE) {
  return {
    status: "pass",
    wikijump_sha: source.wikijump_commit,
    wikijump_tree: source.wikijump_tree,
    ftml_sha: source.ftml_sha,
    images: {
      deepwell: {id: `sha256:${"1".repeat(64)}`},
      framerail: {id: `sha256:${"2".repeat(64)}`},
      wws: {id: `sha256:${"3".repeat(64)}`},
    },
  };
}

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "final-frozen-v2-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  for (const lockfile of LOCKFILES) {
    await fs.mkdir(path.dirname(path.join(root, lockfile)), {recursive: true});
    await fs.writeFile(path.join(root, lockfile), `lock ${lockfile}`);
  }
  const write = async (name, value) => {
    const filePath = path.join(root, "evidence", name);
    await fs.mkdir(path.dirname(filePath), {recursive: true});
    await fs.writeFile(filePath, JSON.stringify(value));
    return filePath;
  };
  return {root, write};
}

test("FINAL_FROZEN CLI takes only executed-evidence inputs", () => {
  const args = parseFinalFrozenArgs([
    "--source-root", "/repo",
    "--validation-receipt", "/evidence/validation.json",
    "--image-producer", "/evidence/image-producer.json",
    "--output", "/evidence/final-frozen.json",
  ]);
  assert.equal(args["validation-receipt"], "/evidence/validation.json");
  assert.throws(() => parseFinalFrozenArgs(["--attestation", "zero-findings-reviewed"]), /unknown or duplicate option/u);
});

test("FINAL_FROZEN binds a passing validation and the sealed images of one source", async (t) => {
  const {root, write} = await fixture(t);
  const receipt = await buildFinalFrozenReceipt({
    source: SOURCE,
    sourceRoot: root,
    validationReceiptPath: await write("validation.json", validation()),
    imageProducerPath: await write("image-producer.json", imageProducer()),
  });
  assert.equal(receipt.status, "FINAL_FROZEN");
  assert.equal(receipt.validation.check_count, 2);
  assert.equal(receipt.source.lockfiles.length, LOCKFILES.length);
  const receiptPath = await write("final-frozen.json", receipt);
  const verified = await verifyFinalFrozenReceipt({receiptPath, source: SOURCE});
  assert.deepEqual(Object.keys(verified.receipt.images.identities), ["deepwell", "framerail", "wws"]);
});

test("FINAL_FROZEN refuses validation that failed, ran nothing, or is for another source", async (t) => {
  const {root, write} = await fixture(t);
  const producer = await write("image-producer.json", imageProducer());
  const build = async (value) => buildFinalFrozenReceipt({
    source: SOURCE,
    sourceRoot: root,
    validationReceiptPath: await write(`validation-${Math.random()}.json`, value),
    imageProducerPath: producer,
  });
  await assert.rejects(
    build(validation({status: "fail", checks: [{name: "deepwell full tests", status: "failed"}]})),
    /did not pass: deepwell full tests/u,
  );
  await assert.rejects(build(validation({checks: []})), /ran no checks/u);
  await assert.rejects(build(validation({wikijump_commit: "f".repeat(40)})), /source identity is stale/u);
  await assert.rejects(build(validation({mode: "checkpoint"})), /final barrier/u);
});

test("FINAL_FROZEN verification rejects changed validation bytes and foreign images", async (t) => {
  const {root, write} = await fixture(t);
  const validationPath = await write("validation.json", validation());
  const receipt = await buildFinalFrozenReceipt({
    source: SOURCE,
    sourceRoot: root,
    validationReceiptPath: validationPath,
    imageProducerPath: await write("image-producer.json", imageProducer()),
  });
  const receiptPath = await write("final-frozen.json", receipt);
  await fs.writeFile(validationPath, JSON.stringify(validation({base_commit: "9".repeat(40)})));
  await assert.rejects(verifyFinalFrozenReceipt({receiptPath, source: SOURCE}), /validation is stale/u);

  await assert.rejects(
    buildFinalFrozenReceipt({
      source: SOURCE,
      sourceRoot: root,
      validationReceiptPath: await write("validation-2.json", validation()),
      imageProducerPath: await write("foreign.json", imageProducer({...SOURCE, wikijump_tree: "0".repeat(40)})),
    }),
    /wikijump_tree does not match the frozen source/u,
  );
});

test("preflight output yields selected groups and every check result", () => {
  const parsed = parsePreflightOutput([
    "preflight: groups: deepwell=true wws=false verification=true ",
    "--- deepwell fmt: ok",
    "--- deepwell full tests: FAILED",
    "preflight: FAILED: deepwell full tests",
  ].join("\n"));
  assert.deepEqual(parsed.groups, {deepwell: true, wws: false, verification: true});
  assert.deepEqual(parsed.checks, [
    {name: "deepwell fmt", status: "ok"},
    {name: "deepwell full tests", status: "failed"},
  ]);
  assert.throws(
    () => validateConsolidatedValidation(validation({checks: parsed.checks, status: "fail"}), SOURCE),
    /did not pass: deepwell full tests/u,
  );
});
