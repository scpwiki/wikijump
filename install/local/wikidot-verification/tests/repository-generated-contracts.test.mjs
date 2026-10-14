import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { verifyTrackedContractConsistency } from "../src/repository-generated-contracts.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const cli = path.join(root, "install/local/wikidot-verification/scripts/verify-repository-generated-contracts.mjs");

function fixture() {
  return {
    inventory: {
      schema: "wikijump.compatibility_surface_inventory.v2",
      counts: { total: 4, by_kind: { deepwell_jsonrpc_method: 1, other: 1, wws_route: 2 } },
      relationship_edge_types: ["implemented_by"],
      owner_keys: { specification: ["spec:a", "spec:deepwell", "spec:wws"], implementation: ["impl:a", "impl:deepwell", "impl:wws"] },
      relationship_edges: [
        { source: "deepwell-jsonrpc:echo", type: "implemented_by", target: "impl:deepwell" },
      ],
      surfaces: [
        { surface_id: "deepwell-jsonrpc:echo", kind: "deepwell_jsonrpc_method", specification_owner: "spec:deepwell", implementation_owners: ["impl:deepwell"] },
        { surface_id: "wws-route:GET:/x", kind: "wws_route", specification_owner: "spec:wws", implementation_owners: ["impl:wws"] },
        { surface_id: "wws-route:HEAD:/x", kind: "wws_route", specification_owner: "spec:wws", implementation_owners: ["impl:wws"] },
        { surface_id: "other:a", kind: "other", specification_owner: "spec:a", implementation_owners: ["impl:a"] },
      ],
    },
    semantics: {
      schema: "wikijump.compatibility_surface_semantics.v1",
      specification_owner_keys: ["spec:a", "spec:deepwell", "spec:wws"],
      implementation_owner_keys: ["impl:a", "impl:deepwell", "impl:wws"],
      implementation_owners_by_legacy_owner: { deepwell: ["impl:deepwell"], other: ["impl:a"], wws: ["impl:wws"] },
      relationship_edge_types: ["implemented_by"],
    },
    deepwellManifest: {
      schema: "wikijump.deepwell_jsonrpc_contract_manifest.v1",
      method_count: 1,
      methods: [{ method: "echo" }],
    },
    wwsDenominator: {
      schema: "wikijump.wws_route_registration_denominator.v2",
      counts: { registrations: 1 },
      registrations: [{ registration_id: "wws-route-registration:GET:/x", path: "/x", declared_method_class: "GET", fallback_handler_symbol: null }],
    },
  };
}

test("repository contract consistency fixes Deepwell, WWS, owner, and edge denominators", () => {
  assert.deepEqual(verifyTrackedContractConsistency(fixture()), {
    surface_count: 4,
    deepwell_method_count: 1,
    wws_surface_count: 2,
    specification_owner_count: 3,
    implementation_owner_count: 3,
    relationship_edge_type_count: 1,
  });
});

test("repository contract consistency rejects stale generated and orphan owner state", () => {
  const staleDeepwell = fixture();
  staleDeepwell.deepwellManifest.methods.push({ method: "new_method" });
  staleDeepwell.deepwellManifest.method_count = 2;
  assert.throws(() => verifyTrackedContractConsistency(staleDeepwell), /Deepwell inventory surface denominator drift/u);

  const staleWws = fixture();
  staleWws.wwsDenominator.registrations[0].fallback_handler_symbol = "fallback";
  assert.throws(() => verifyTrackedContractConsistency(staleWws), /WWS inventory surface denominator drift/u);

  const orphanOwner = fixture();
  orphanOwner.semantics.implementation_owner_keys.push("impl:orphan");
  orphanOwner.semantics.implementation_owner_keys.sort();
  orphanOwner.inventory.owner_keys.implementation.push("impl:orphan");
  orphanOwner.inventory.owner_keys.implementation.sort();
  assert.throws(() => verifyTrackedContractConsistency(orphanOwner), /used implementation owner denominator drift/u);

  const staleCounts = fixture();
  staleCounts.inventory.counts.total = 3;
  assert.throws(() => verifyTrackedContractConsistency(staleCounts), /total count is stale/u);
});

test("WWS repository consistency distinguishes ANY-only from GET plus fallback dispatch", () => {
  const value = fixture();
  value.inventory.counts = { total: 6, by_kind: { deepwell_jsonrpc_method: 1, other: 1, wws_route: 4 } };
  value.inventory.surfaces.splice(3, 0,
    { surface_id: "wws-route:FALLBACK:/x", kind: "wws_route", specification_owner: "spec:wws", implementation_owners: ["impl:wws"] },
    { surface_id: "wws-route:ANY:/y", kind: "wws_route", specification_owner: "spec:wws", implementation_owners: ["impl:wws"] },
  );
  value.wwsDenominator.counts.registrations = 3;
  value.wwsDenominator.registrations = [
    { registration_id: "wws-route-registration:GET:/x", path: "/x", declared_method_class: "GET", fallback_handler_symbol: null },
    { registration_id: "wws-route-registration:ANY:/x", path: "/x", declared_method_class: "ANY", fallback_handler_symbol: null },
    { registration_id: "wws-route-registration:ANY:/y", path: "/y", declared_method_class: "ANY", fallback_handler_symbol: null },
  ];
  assert.equal(verifyTrackedContractConsistency(value).wws_surface_count, 4);

  value.wwsDenominator.registrations[1] = { ...value.wwsDenominator.registrations[1], path: "/z", registration_id: "wws-route-registration:ANY:/z" };
  assert.throws(() => verifyTrackedContractConsistency(value), /WWS inventory surface denominator drift/u);
});

test("repository-only generated contract CLI is package-install-free and passes on the tracked repository", () => {
  const result = spawnSync(process.execPath, [cli, "--root", root], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(readFileSync(path.join(root, "docs/development/deepwell-jsonrpc-contract-manifest.json"), "utf8"));
  assert.match(result.stdout, new RegExp(`verified ${manifest.method_count} Deepwell JSON-RPC contracts`, "u"));
  assert.match(result.stdout, /verified 34 WWS route registrations/u);
  assert.match(result.stdout, /verified repository generated-contract consistency/u);
});

test("full generated contract CLI regenerates v3 inventory from exact local HEAD", () => {
  const result = spawnSync(process.execPath, [cli, "--root", root, "--full"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /verified full compatibility inventory bytes at [0-9a-f]{40}/u);
});
