import assert from "node:assert/strict";
import { test } from "node:test";

import { inspectBrowserCaptureBarriers } from "../scripts/inspect-browser-capture-barriers.mjs";

const cacheIdentity = "sealed-anonymous-fixture";
const matchingUrl = "https://example.org/asset.js?token=supersecret";
const otherUrl =
  "https://other.example.net/not-cached.png?authorization=secret2";
function records({ barrierCount = 1, failed = true } = {}) {
  return {
    schema: "wikijump_full_parity.browser_rendering_evidence.v1",
    capture: {
      source_response_cache: {
        persistent_identity: cacheIdentity,
        acquisition_barriers: barrierCount,
      },
      request_gate: { enforcement_failed: failed },
    },
    evidence: [
      {
        schema: "wikijump_full_parity.browser_rendering_record.v1",
        source_failed_requests: [
          { url: matchingUrl, failure: "net::ERR_BLOCKED_BY_CLIENT" },
          { url: otherUrl, failure: "net::ERR_ABORTED" },
        ],
        capture_errors: [],
      },
    ],
  };
}
function manifest({ barriers = [matchingUrl] } = {}) {
  return {
    schema: "wikijump_full_parity.browser_response_cache_store.v1",
    identity: cacheIdentity,
    entries: [],
    acquisition_barriers: barriers,
  };
}
const lock = {
  schema: "wikijump_full_parity.browser_capture_lock.v1",
  state_confirmation: "pending",
  run_id: "run-1",
  hostname: "localhost",
  pid: 10,
  process_start_ticks: "123",
};
const gateConfig = {
  schema: "wikijump_full_parity.browser_request_gate_config.v1",
  run_id: "run-1",
  lock: { owner: { ...lock } },
};

test("offline inspection correlates exact failed URLs without reporting path, query, or credentials", () => {
  const report = inspectBrowserCaptureBarriers(
    records(),
    manifest(),
    lock,
    gateConfig,
  );
  assert.equal(report.diagnostic_only, true);
  assert.equal(report.compatible_oracle_proven, false);
  assert.equal(report.source_failed_requests, 2);
  assert.equal(report.source_failed_requests_matching_current_barriers, 1);
  assert.deepEqual(report.matched_hosts, ["example.org"]);
  assert.equal(report.barrier_counts_disagree, false);
  assert.equal(report.gate_enforcement_failed, true);
  assert.equal(report.lock_state_confirmation, "pending");
  assert.doesNotMatch(
    JSON.stringify(report),
    /supersecret|secret2|asset\.js|not-cached\.png|authorization|token=/u,
  );
});

test("changed current manifest count is explicitly not a contemporaneous barrier census", () => {
  const report = inspectBrowserCaptureBarriers(
    records(),
    manifest({ barriers: [matchingUrl, otherUrl] }),
  );
  assert.equal(report.barrier_counts_disagree, true);
  assert.equal(report.source_failed_requests_matching_current_barriers, 2);
  assert.match(report.attribution_limit, /not a time-frozen proof/u);
});

test("empty current barrier inventory is a valid diagnostic rather than a compatibility pass", () => {
  const input = manifest();
  delete input.acquisition_barriers;
  const report = inspectBrowserCaptureBarriers(
    records({ barrierCount: 0, failed: false }),
    input,
  );
  assert.equal(report.current_cache_acquisition_barriers, 0);
  assert.equal(report.source_failed_requests_matching_current_barriers, 0);
  assert.equal(report.compatible_oracle_proven, false);
});

test("offline inspector fails closed on mismatched cache identity", () => {
  const input = manifest();
  input.identity = "different-campaign";
  assert.throws(
    () => inspectBrowserCaptureBarriers(records(), input),
    /identities differ/u,
  );
});

test("offline inspector rejects malformed sources and credentials without printing raw URLs", () => {
  const malformed = records();
  malformed.evidence[0].source_failed_requests[0].url = "file:///etc/passwd";
  assert.throws(
    () => inspectBrowserCaptureBarriers(malformed, manifest()),
    /anonymous HTTP\(S\) URL/u,
  );
  const credentialed = manifest({
    barriers: ["https://user:password@example.org/foo"],
  });
  assert.throws(
    () => inspectBrowserCaptureBarriers(records(), credentialed),
    /anonymous HTTP\(S\) URL/u,
  );
});

test("offline inspector rejects absent gate, inconsistent capture and invalid lock", () => {
  const noGate = records();
  delete noGate.capture.request_gate;
  assert.throws(
    () => inspectBrowserCaptureBarriers(noGate, manifest()),
    /request gate result/u,
  );
  const invalid = records();
  invalid.evidence[0].schema = "unknown";
  assert.throws(
    () => inspectBrowserCaptureBarriers(invalid, manifest()),
    /evidence rows are malformed/u,
  );
  assert.throws(
    () =>
      inspectBrowserCaptureBarriers(
        records(),
        manifest(),
        { schema: "wrong", state_confirmation: "sealed" },
        gateConfig,
      ),
    /lock does not match/u,
  );
  assert.throws(
    () => inspectBrowserCaptureBarriers(records(), manifest(), lock),
    /must be inspected together/u,
  );
  assert.throws(
    () =>
      inspectBrowserCaptureBarriers(
        records(),
        manifest(),
        { ...lock, run_id: "different-run" },
        gateConfig,
      ),
    /lock does not match/u,
  );
});
