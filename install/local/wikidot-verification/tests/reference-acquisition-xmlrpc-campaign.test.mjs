import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildWikidotXmlrpcCampaign,
  openWikidotXmlrpcCampaign,
  parseWikidotXmlrpcCampaign,
  putWikidotXmlrpcCampaign,
  serializeWikidotXmlrpcCampaign,
} from "../src/reference-acquisition-xmlrpc-campaign.mjs";
import {
  buildWikidotXmlrpcImplementation,
  putWikidotXmlrpcImplementation,
} from "../src/reference-acquisition-xmlrpc-implementation.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

const INVENTORY = "b".repeat(64);
const NONCE = "00000000-0000-4000-8000-000000000001";
const REFERENCE = { algorithm: "sha256", bytes: 1, sha256: "e".repeat(64) };

function implementation() {
  return buildWikidotXmlrpcImplementation({
    coordinatorFileSha256: "a".repeat(64),
    dependencyLockFileSha256: "c".repeat(64),
    nodeVersion: "v26.4.0",
    pythonVersion: "3.14.0",
    wikijumpCommit: "1".repeat(40),
    wikijumpTree: "2".repeat(40),
    workerFileSha256: "d".repeat(64),
    workerRepositoryCommit: "3".repeat(40),
    workerRepositoryTree: "4".repeat(40),
  });
}

function options(implementationReference = REFERENCE, overrides = {}) {
  return {
    campaignNonce: NONCE,
    implementation: implementationReference,
    inventorySha256: INVENTORY,
    principalId: 5700026,
    ...overrides,
  };
}

function campaign(implementationReference = REFERENCE, overrides = {}) {
  return buildWikidotXmlrpcCampaign(
    options(implementationReference, overrides),
  );
}

async function fixture(t) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-campaign-"));
  const state = { root: path.join(parent, "store"), store: undefined };
  state.store = await initializeReferenceObjectStore(state.root);
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

function objectPath(root, reference) {
  return path.join(
    root,
    ...referenceObjectRelativePath(reference.sha256).split("/"),
  );
}

function referenceFor(bytes) {
  return {
    algorithm: "sha256",
    bytes: bytes.length,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  };
}

function assertSecretSafe(call, secret) {
  assert.throws(call, (error) => !error.message.includes(secret));
}

test("schema and canonical bytes bind the complete campaign authority", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-campaign-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = campaign();
  assert.deepEqual(Object.keys(value).sort(), schema.required);
  assert.deepEqual(
    Object.keys(value).sort(),
    Object.keys(schema.properties).sort(),
  );
  assert.deepEqual(
    parseWikidotXmlrpcCampaign(serializeWikidotXmlrpcCampaign(value)),
    value,
  );
  const { campaign_id: campaignId, ...body } = value;
  assert.equal(
    campaignId,
    crypto.createHash("sha256").update(stableStringify(body)).digest("hex"),
  );
  assert(Object.isFrozen(value) && Object.isFrozen(value.implementation));
});

test("campaign identity survives restart and nonce recapture changes every identity", async (t) => {
  const state = await fixture(t);
  let { store } = state;
  const storedImplementation = await putWikidotXmlrpcImplementation(
    store,
    implementation(),
  );
  const first = await putWikidotXmlrpcCampaign(
    store,
    campaign(storedImplementation.object),
  );
  const repeated = await putWikidotXmlrpcCampaign(
    store,
    campaign(storedImplementation.object),
  );
  assert.equal(first.disposition, "created");
  assert.equal(repeated.disposition, "exists");
  assert.deepEqual(repeated.producer, first.producer);
  await store.close();
  store = await openReferenceObjectStore(state.root);
  state.store = store;
  const reopened = await openWikidotXmlrpcCampaign(store, first.reference, {
    expectedInventorySha256: INVENTORY,
  });
  assert.deepEqual(reopened.producer, first.producer);
  assert.deepEqual(reopened.implementation, implementation());
  const recapture = await putWikidotXmlrpcCampaign(
    store,
    campaign(storedImplementation.object, {
      campaignNonce: "00000000-0000-4000-8000-000000000002",
    }),
  );
  assert.notEqual(
    recapture.descriptor.campaign_id,
    first.descriptor.campaign_id,
  );
  assert.notEqual(recapture.reference.sha256, first.reference.sha256);
  assert.notDeepEqual(recapture.producer, first.producer);
  assert.throws(
    () =>
      serializeWikidotXmlrpcCampaign({ ...first.descriptor, principal_id: 1 }),
    /campaign ID/u,
  );
});

test("authority and hostile inputs fail closed without leaking values", () => {
  const value = campaign();
  for (const [field, changed] of [
    ["endpoint", "https://example.invalid"],
    ["layer", "http_document"],
    ["principal_id", Number.MAX_SAFE_INTEGER + 1],
    ["read_only", false],
  ]) {
    assert.throws(() =>
      serializeWikidotXmlrpcCampaign({ ...value, [field]: changed }),
    );
  }
  const secret = "sentinel-secret-must-not-survive";
  const accessor = options();
  Object.defineProperty(accessor, "campaignNonce", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxy = new Proxy(options(), {
    ownKeys() {
      throw new Error(secret);
    },
  });
  const coercible = {
    valueOf() {
      throw new Error(secret);
    },
  };
  const nestedAccessor = { ...REFERENCE };
  Object.defineProperty(nestedAccessor, "algorithm", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const nestedProxy = new Proxy(REFERENCE, {
    ownKeys() {
      throw new Error(secret);
    },
  });
  const nestedSymbol = { ...REFERENCE, [Symbol("secret")]: secret };
  const symbol = { ...value, [Symbol("secret")]: secret };
  for (const call of [
    () => buildWikidotXmlrpcCampaign(accessor),
    () => buildWikidotXmlrpcCampaign(proxy),
    () => buildWikidotXmlrpcCampaign(options(nestedAccessor)),
    () => buildWikidotXmlrpcCampaign(options(nestedProxy)),
    () => buildWikidotXmlrpcCampaign(options(nestedSymbol)),
    () => parseWikidotXmlrpcCampaign(coercible),
    () => serializeWikidotXmlrpcCampaign({ ...value, api_key: secret }),
    () => serializeWikidotXmlrpcCampaign(symbol),
  ]) {
    assertSecretSafe(call, secret);
  }
  const reversed = Object.fromEntries(Object.entries(value).reverse());
  for (const bytes of [
    Buffer.from([0xff, 0x0a]),
    Buffer.from("{}\r\n"),
    Buffer.from(`${JSON.stringify(reversed)}\n`),
    Buffer.alloc(16 * 1024 + 1, 0x20),
  ]) {
    assert.throws(() => parseWikidotXmlrpcCampaign(bytes));
  }
});

test("publication and opening verify inventory and transitive implementation CAS", async (t) => {
  const state = await fixture(t);
  const { store } = state;
  const missing = { algorithm: "sha256", bytes: 1, sha256: "f".repeat(64) };
  const dangling = campaign(missing);
  await assert.rejects(putWikidotXmlrpcCampaign(store, dangling), /ENOENT/u);
  const danglingBytes = serializeWikidotXmlrpcCampaign(dangling);
  await assert.rejects(
    store.readObject(referenceFor(danglingBytes), {
      maxBytes: danglingBytes.length,
    }),
    /ENOENT/u,
  );
  const storedImplementation = await putWikidotXmlrpcImplementation(
    store,
    implementation(),
  );
  const storedCampaign = await putWikidotXmlrpcCampaign(
    store,
    campaign(storedImplementation.object),
  );
  await assert.rejects(
    openWikidotXmlrpcCampaign(store, storedCampaign.reference, {
      expectedInventorySha256: "0".repeat(64),
    }),
    /wrong inventory/u,
  );
  const secret = "sentinel-open-secret";
  const proxy = new Proxy(
    { expectedInventorySha256: INVENTORY },
    {
      ownKeys() {
        throw new Error(secret);
      },
    },
  );
  await assert.rejects(
    openWikidotXmlrpcCampaign(store, storedCampaign.reference, proxy),
    (error) => !error.message.includes(secret),
  );
  const referenceProxy = new Proxy(storedCampaign.reference, {
    ownKeys() {
      throw new Error(secret);
    },
  });
  const referenceAccessor = { ...storedCampaign.reference };
  Object.defineProperty(referenceAccessor, "algorithm", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const referenceSymbol = {
    ...storedCampaign.reference,
    [Symbol("secret")]: secret,
  };
  for (const reference of [
    referenceAccessor,
    referenceProxy,
    referenceSymbol,
  ]) {
    await assert.rejects(
      openWikidotXmlrpcCampaign(store, reference, {
        expectedInventorySha256: INVENTORY,
      }),
      (error) => !error.message.includes(secret),
    );
  }
  const implementationPath = objectPath(
    state.root,
    storedImplementation.object,
  );
  await fs.chmod(implementationPath, 0o600);
  await fs.writeFile(
    implementationPath,
    Buffer.alloc(storedImplementation.object.bytes, 0x20),
  );
  await fs.chmod(implementationPath, 0o400);
  await assert.rejects(
    openWikidotXmlrpcCampaign(store, storedCampaign.reference, {
      expectedInventorySha256: INVENTORY,
    }),
    /corrupt/u,
  );
});
