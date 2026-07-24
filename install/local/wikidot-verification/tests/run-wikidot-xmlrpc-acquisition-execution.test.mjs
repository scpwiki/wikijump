import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  assertDistinctOutputDestinations,
  capturePending,
  expectedWorkerExitCode,
  normalizeRunnerOptions,
  partitionRunnerOptions,
  runAcquisition,
} from "../src/wikidot-xmlrpc-acquisition-runner.mjs";
import {
  createAcquisitionFixture,
  createXmlrpcCampaignFixture,
  responseFor,
} from "./wikidot-xmlrpc-acquisition-fixtures.mjs";
import {runnerOptions} from "./support/run-wikidot-xmlrpc-acquisition-fixture.mjs";

test("runner rejects output destination aliases before a throttle can be sealed", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-output-"));
  t.after(() => fs.rm(root, { force: true, recursive: true }));
  const options = partitionRunnerOptions(normalizeRunnerOptions(
    runnerOptions({
      "inventory-output": path.join(root, "inventory.json"),
      "result-receipt": path.join(root, "result.json"),
      "throttle-receipt": path.join(root, "throttle.json"),
      verdict: path.join(root, "verdict.json"),
    }),
  )).outputs;
  await assert.doesNotReject(assertDistinctOutputDestinations(options));
  await assert.rejects(
    assertDistinctOutputDestinations({
      ...options,
      resultReceipt: options.throttleReceipt,
    }),
    /output_destinations_alias/u,
  );
  await fs.writeFile(options.inventoryOutput, "receipt\n");
  await fs.link(options.inventoryOutput, options.resultReceipt);
  await assert.rejects(
    assertDistinctOutputDestinations(options),
    /output_destinations_alias/u,
  );
});

test("coordinator rejects a direct mutable-checkout launch before opening inputs", async () => {
  await assert.rejects(
    runAcquisition(runnerOptions()),
    /materialized_launch_descriptor_unavailable/u,
  );
});

test("retryable and internal worker outcomes require their declared worker exits", () => {
  assert.equal(
    expectedWorkerExitCode({ code: "transport_exhausted", retryable: true }),
    75,
  );
  assert.equal(
    expectedWorkerExitCode({ code: "worker_internal_error", retryable: false }),
    70,
  );
  assert.equal(
    expectedWorkerExitCode({ code: "wikidot_forbidden", retryable: false }),
    null,
  );
});

test("offline worker capture publishes semantic completion without exposing a launch capability", async (t) => {
  const state = await createAcquisitionFixture(t, 1);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const calls = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture(ordinal, fullname) {
        calls.push({ fullname, ordinal });
        return { ok: true, response: responseFor(state, ordinal) };
      },
      async expectExit() {
        assert.fail("successful worker must remain available until clean EOF");
      },
    },
  });
  assert.deepEqual(calls, [{ fullname: "scp-173", ordinal: 0 }]);
  assert.deepEqual(outcome, {
    failure: null,
    status: "complete",
    workerExited: false,
  });
  assert.equal((await semantic.planResume()).pending.length, 0);
});

test("exact deleted worker results become tombstones and later captures continue", async (t) => {
  const state = await createAcquisitionFixture(t, 3);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const calls = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture(ordinal, fullname) {
        calls.push({ fullname, ordinal });
        if (ordinal === 1) {
          return {
            code: "wikidot_deleted",
            ok: false,
            ordinal,
            retryable: false,
          };
        }
        return { ok: true, response: responseFor(state, ordinal) };
      },
      async expectExit() {
        assert.fail("deleted page does not terminate a healthy worker");
      },
    },
  });
  assert.deepEqual(calls, [
    { fullname: "scp-173", ordinal: 0 },
    { fullname: "scp-174", ordinal: 1 },
    { fullname: "scp-175", ordinal: 2 },
  ]);
  assert.deepEqual(outcome, {
    failure: null,
    status: "complete",
    workerExited: false,
  });
  const plan = await semantic.planResume();
  assert.equal(plan.pending.length, 0);
  assert.equal(plan.complete.length, 3);
  assert.equal((await semantic.resolve({ ordinal: 0 })).kind, "live");
  const deleted = await semantic.resolve({ ordinal: 1 });
  assert.equal(deleted.kind, "deleted");
  assert.equal(deleted.tombstone.classification, "wikidot_deleted");
  assert.equal("response" in deleted, false);
  assert.equal((await semantic.resolve({ ordinal: 2 })).kind, "live");
});

test("forbidden and unclassified worker results remain terminal and pending", async (t) => {
  for (const code of ["wikidot_forbidden", "wikidot_fault_unclassified"]) {
    await t.test(code, async (t) => {
      const state = await createAcquisitionFixture(t, 2);
      const { semantic } = await createXmlrpcCampaignFixture(state);
      const calls = [];
      const outcome = await capturePending({
        completions: semantic,
        context: state.context,
        store: state.store,
        worker: {
          async capture(ordinal) {
            calls.push(ordinal);
            return { code, ok: false, ordinal, retryable: false };
          },
          async expectExit() {
            assert.fail("terminal fault should not require worker exit");
          },
        },
      });
      assert.deepEqual(calls, [0]);
      assert.equal(outcome.status, "terminal_stop");
      assert.equal(outcome.failure.code, code);
      assert.equal((await semantic.planResume()).pending.length, 2);
      assert.equal(await semantic.resolve({ ordinal: 0 }), null);
    });
  }
});

test("offline retryable capture persists the failed attempt and requires exit 75", async (t) => {
  const state = await createAcquisitionFixture(t, 1);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const exits = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture() {
        return {
          code: "transport_exhausted",
          ok: false,
          ordinal: 0,
          retryable: true,
        };
      },
      async expectExit(code) {
        exits.push(code);
      },
    },
  });
  assert.deepEqual(exits, [75]);
  assert.equal(outcome.status, "retryable_stop");
  assert.equal(outcome.failure.code, "transport_exhausted");
  assert.equal(outcome.failure.retryable, true);
  assert.equal(outcome.failure.attempt.algorithm, "sha256");
  await state.store.verifyObject(outcome.failure.attempt);
  assert.equal((await semantic.planResume()).pending.length, 1);
});
