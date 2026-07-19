import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import { createReferenceAcquisitionContext } from "../src/reference-acquisition-attempt.mjs";
import {
  buildWikidotXmlrpcCampaign,
  putWikidotXmlrpcCampaign,
} from "../src/reference-acquisition-xmlrpc-campaign.mjs";
import {
  parseWikidotXmlrpcAcquisitionVerdict,
  publishWikidotXmlrpcAcquisitionVerdict,
  WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA,
} from "../src/wikidot-xmlrpc-acquisition-verdict.mjs";
import {
  buildInventory,
  completeDeletedXmlrpcOrdinal,
  completeXmlrpcOrdinal,
  createAcquisitionFixture,
  createXmlrpcCampaignFixture,
  PRINCIPAL_ID,
} from "./wikidot-xmlrpc-acquisition-fixtures.mjs";

function publicationOptions(state, campaignReference) {
  return {
    campaignReference,
    context: state.context,
    store: state.store,
  };
}

test("final verdict is derived from exact semantic campaign completion, not a shaped run result", async (t) => {
  const state = await createAcquisitionFixture(t);
  const { campaign, implementation } = await createXmlrpcCampaignFixture(state);
  await completeXmlrpcOrdinal(state, campaign, 0);
  await completeXmlrpcOrdinal(state, campaign, 1);
  await state.semantic.close();
  state.semantic = undefined;

  const output = path.join(state.receiptDirectory, "final.verdict.json");
  const first = await publishWikidotXmlrpcAcquisitionVerdict(
    output,
    publicationOptions(state, campaign.reference),
  );
  const second = await publishWikidotXmlrpcAcquisitionVerdict(
    output,
    publicationOptions(state, campaign.reference),
  );
  const parsed = parseWikidotXmlrpcAcquisitionVerdict(first.bytes);
  assert.equal(first.disposition, "created");
  assert.equal(second.disposition, "exists");
  assert.deepEqual(first.bytes, second.bytes);
  assert.deepEqual(parsed, {
    campaign: campaign.reference,
    completed: 2,
    implementation: implementation.object,
    schema: WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA,
    status: "pass",
  });
  assert.equal(first.bytes.includes(Buffer.from("2026-07-19")), false);
  assert.equal(first.bytes.includes(Buffer.from("/home/")), false);
  await fs.chmod(output, 0o600);
  await fs.writeFile(output, "{}\n");
  await fs.chmod(output, 0o400);
  await assert.rejects(
    publishWikidotXmlrpcAcquisitionVerdict(
      output,
      publicationOptions(state, campaign.reference),
    ),
    /conflicts/u,
  );
});

test("final verdict counts deleted tombstones as semantically resolved targets", async (t) => {
  const state = await createAcquisitionFixture(t);
  const { campaign } = await createXmlrpcCampaignFixture(state);
  await completeXmlrpcOrdinal(state, campaign, 0);
  await completeDeletedXmlrpcOrdinal(state, campaign, 1);
  await state.semantic.close();
  state.semantic = undefined;

  const output = path.join(state.receiptDirectory, "mixed.verdict.json");
  const publication = await publishWikidotXmlrpcAcquisitionVerdict(
    output,
    publicationOptions(state, campaign.reference),
  );
  const parsed = parseWikidotXmlrpcAcquisitionVerdict(publication.bytes);
  assert.equal(parsed.status, "pass");
  assert.equal(parsed.completed, 2);
});

test("incomplete, wrong-campaign, wrong-inventory, and shaped inputs cannot publish a final verdict", async (t) => {
  const state = await createAcquisitionFixture(t);
  const { campaign, implementation } = await createXmlrpcCampaignFixture(state);
  await completeXmlrpcOrdinal(state, campaign, 0);
  const output = path.join(state.receiptDirectory, "incomplete.verdict.json");
  await assert.rejects(
    publishWikidotXmlrpcAcquisitionVerdict(
      output,
      publicationOptions(state, campaign.reference),
    ),
    /requires every campaign target/u,
  );
  await assert.rejects(fs.access(output));
  await completeXmlrpcOrdinal(state, campaign, 1);

  const otherCampaign = await putWikidotXmlrpcCampaign(
    state.store,
    buildWikidotXmlrpcCampaign({
      campaignNonce: "00000000-0000-4000-8000-000000000002",
      implementation: implementation.object,
      inventorySha256: state.inventory.identity.sha256,
      principalId: PRINCIPAL_ID,
    }),
  );
  await assert.rejects(
    publishWikidotXmlrpcAcquisitionVerdict(
      output,
      publicationOptions(state, otherCampaign.reference),
    ),
    /requires every campaign target/u,
  );
  const wrongInventory = buildInventory(1);
  await assert.rejects(
    publishWikidotXmlrpcAcquisitionVerdict(output, {
      campaignReference: campaign.reference,
      context: createReferenceAcquisitionContext(wrongInventory, {
        expectedIdentitySha256: wrongInventory.identity.sha256,
      }),
      store: state.store,
    }),
    /wrong inventory identity/u,
  );
  await assert.rejects(
    publishWikidotXmlrpcAcquisitionVerdict(output, {
      ...publicationOptions(state, campaign.reference),
      completed: 24430,
      status: "pass",
    }),
    /unexpected fields/u,
  );
});

test("verdict parser rejects malformed, noncanonical, and unexpected inputs without echoing values", () => {
  const sentinel = "sentinel-verdict-value";
  for (const action of [
    () => parseWikidotXmlrpcAcquisitionVerdict(Buffer.from("{}\n")),
    () => parseWikidotXmlrpcAcquisitionVerdict(Buffer.from("{}\r\n")),
    () => parseWikidotXmlrpcAcquisitionVerdict(Buffer.from([0xff, 0x0a])),
    () =>
      parseWikidotXmlrpcAcquisitionVerdict(
        Buffer.from(`{"unexpected":"${sentinel}"}\n`),
      ),
  ]) {
    assert.throws(action, (error) => !error.message.includes(sentinel));
  }
});
