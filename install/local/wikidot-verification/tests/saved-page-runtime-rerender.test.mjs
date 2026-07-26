import assert from "node:assert/strict";
import test from "node:test";

import {
  rerenderSavedPageRuntime,
  validateSavedPageRerenderReceipt,
} from "../src/saved-page-runtime-rerender.mjs";
import {sha256} from "../src/syntax-differential.mjs";

const runtimeIdentity = {
  schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1",
  wikijump_sha: "1".repeat(40),
  ftml_sha: "2".repeat(40),
  dependency_lock_sha256: "3".repeat(64),
  executable_sha256: "4".repeat(64),
  runtime_config_sha256: "5".repeat(64),
};

function reference(source = "saved source") {
  const selectedHtml = '<div class="anom-bar-container item-9507-D clear-3">ok</div>';
  return {
    schema: "wikijump_syntax_differential.wikidot_saved_page_reference.v1",
    case: {
      case_id: "scp-9507-stray-open-include",
      selector: ".anom-bar-container",
      expected: {
        required_class_tokens: ["anom-bar-container", "item-9507-D", "clear-3"],
        forbidden_literals: ["[[include", "[["],
      },
    },
    actor: {authenticated: false},
    site: {unix_name: "scp-wiki", domain: "scp-wiki.wikidot.com"},
    page: {
      slug: "scp-9507",
      source_wikitext: source,
      source_sha256: sha256(source),
    },
    selected_html: selectedHtml,
    selected_html_sha256: sha256(selectedHtml),
    provenance: {
      transport: "anonymous-https",
      mutated: false,
      wikidot_py_commit: "6".repeat(40),
      requirements_sha256: "7".repeat(64),
      requirements_lock_sha256: "8".repeat(64),
    },
  };
}

function page(generator, source = "saved source") {
  return {
    page_id: 10,
    page_category_id: 11,
    revision_id: 12,
    site_id: 13,
    slug: "scp-9507",
    wikitext: source,
    compiled_at: "2026-07-27T00:00:00Z",
    compiled_generator: generator,
  };
}

class FakeRpc {
  constructor({after = page("ftml [22222222]; deepwell-render/v1")} = {}) {
    this.after = after;
    this.calls = [];
    this.rerendered = false;
  }

  async call(method, params, context = {}) {
    this.calls.push({method, params, context});
    if (method === "ping") return "Pong!";
    if (method === "site_get") return {site_id: 13};
    if (method === "login") return {needs_mfa: false, session_token: "token"};
    if (method === "session_get") return {user_id: 14};
    if (method === "page_get") {
      return this.rerendered ? this.after : page("ftml [11111111]; deepwell-render/v1");
    }
    if (method === "page_rerender") {
      this.rerendered = true;
      return null;
    }
    throw new Error(`unexpected method ${method}`);
  }
}

test("saved-page rerender preserves source and revision while updating the compiler", async () => {
  const rpc = new FakeRpc();
  const times = ["2026-07-27T00:00:00Z", "2026-07-27T00:00:01Z"];
  const receipt = await rerenderSavedPageRuntime({
    references: [reference()],
    runtimeIdentity,
    administratorEmail: "admin@example.test",
    administratorPassword: "secret",
    rpcClient: rpc,
    now: () => times.shift(),
  });
  assert.equal(receipt.status, "pass");
  assert.equal(receipt.pages[0].before.revision_id, receipt.pages[0].after.revision_id);
  assert.equal(receipt.pages[0].after.compiled_generator, "ftml [22222222]; deepwell-render/v1");
  assert.deepEqual(
    rpc.calls.find((call) => call.method === "page_rerender").params,
    {site_id: 13, category_id: 11, page_id: 10},
  );
  assert.equal(
    validateSavedPageRerenderReceipt(receipt, [reference()], runtimeIdentity),
    receipt,
  );
});

test("saved-page rerender refuses a local source that differs from the frozen Wikidot page", async () => {
  const rpc = new FakeRpc();
  rpc.call = async (method) => {
    if (method === "ping") return "Pong!";
    if (method === "site_get") return {site_id: 13};
    if (method === "login") return {needs_mfa: false, session_token: "token"};
    if (method === "session_get") return {user_id: 14};
    if (method === "page_get") return page("ftml [11111111]", "different source");
    throw new Error("mutation must not be reached");
  };
  await assert.rejects(
    rerenderSavedPageRuntime({
      references: [reference()],
      runtimeIdentity,
      administratorEmail: "admin@example.test",
      administratorPassword: "secret",
      rpcClient: rpc,
    }),
    /source differs/u,
  );
});

test("saved-page rerender rejects a stale compiler after rerender", async () => {
  await assert.rejects(
    rerenderSavedPageRuntime({
      references: [reference()],
      runtimeIdentity,
      administratorEmail: "admin@example.test",
      administratorPassword: "secret",
      rpcClient: new FakeRpc({after: page("ftml [11111111]; deepwell-render/v1")}),
    }),
    /was not compiled by FTML 22222222/u,
  );
});

test("rerender receipt validation rejects a different source-bound case set", async () => {
  const receipt = await rerenderSavedPageRuntime({
    references: [reference()],
    runtimeIdentity,
    administratorEmail: "admin@example.test",
    administratorPassword: "secret",
    rpcClient: new FakeRpc(),
  });
  const changed = structuredClone(receipt);
  changed.pages[0].after.source_sha256 = "9".repeat(64);
  assert.throws(
    () => validateSavedPageRerenderReceipt(changed, [reference()], runtimeIdentity),
    /source identity differs/u,
  );
});
