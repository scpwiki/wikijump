import assert from "node:assert/strict";
import test from "node:test";

import {
  LISTPAGES_LIVE_FIXTURE_CAPTURE_SCHEMA,
  LISTPAGES_LIVE_FIXTURE_CLASSIFICATION_SCHEMA,
  classifyListPagesLiveFixtures,
  extractListPagesCaseBlocks,
  validateLiveFixtureCapture,
} from "../src/listpages-live-fixture-classification.mjs";
import {sha256} from "../src/syntax-differential.mjs";

function capture(pageContentHtml) {
  const rawPageHtml = `<html><body>${pageContentHtml}</body></html>`;
  return {
    schema: LISTPAGES_LIVE_FIXTURE_CAPTURE_SCHEMA,
    captured_at: "2026-07-27T00:00:00Z",
    case: {
      case_id: "lp-live-sample",
      dimensions: ["pagination"],
    },
    request: {
      url: "http://sandbox-for-codex.wikidot.com/run-owned:lp-campaign-20260727-sample/p/2",
      method: "GET",
      authenticated: false,
      status: 200,
    },
    site: {
      unix_name: "sandbox-for-codex",
      domain: "sandbox-for-codex.wikidot.com",
    },
    fixture_graph: [],
    provenance: {
      mutated: true,
    },
    capture_status: "captured",
    raw_page_html: rawPageHtml,
    raw_page_html_sha256: sha256(rawPageHtml),
    page_content_html: pageContentHtml,
    page_content_html_sha256: sha256(pageContentHtml),
  };
}

test("extracts ListPages case rows and pager links from Wikidot HTML", () => {
  const blocks = extractListPagesCaseBlocks(`
    <div id="page-content">
      <div class="lp-case lp-pagination-five">
        <p>PAGINATION FIVE</p>
        <div class="list-pages-box">
          <p>F6:lp-campaign-20260727-page-06|<br/>F7:lp-campaign-20260727-page-07|</p>
          <div class="pager">
            <span class="pager-no">page 2 of 5</span>
            <span class="target"><a href="/sample/p/1">&laquo; previous</a></span>
            <span class="current">2</span>
            <span class="target"><a href="/sample/p/3">next &raquo;</a></span>
          </div>
        </div>
      </div>
    </div>
  `);
  assert.equal(blocks.length, 1);
  assert.equal(blocks[0].block_class, "lp-pagination-five");
  assert.deepEqual(blocks[0].rows, [
    "F6:lp-campaign-20260727-page-06",
    "F7:lp-campaign-20260727-page-07",
  ]);
  assert.deepEqual(blocks[0].page_names, [
    "lp-campaign-20260727-page-06",
    "lp-campaign-20260727-page-07",
  ]);
  assert.equal(blocks[0].pager[0].pager_no[0], "page 2 of 5");
  assert.deepEqual(blocks[0].pager[0].current, ["2"]);
  assert.deepEqual(blocks[0].pager[0].links.map((link) => link.href), [
    "/sample/p/1",
    "/sample/p/3",
  ]);
});

test("classifies captures and carries live environment blockers", () => {
  const pageContentHtml = `
    <div id="page-content">
      <div class="lp-case lp-a"><div class="list-pages-box"><p>lp-campaign-20260727-a|</p></div></div>
      <div class="lp-case lp-b"><div class="list-pages-box"></div></div>
    </div>
  `;
  const blocker = {
    capability: "nonzero rating and vote mutation",
    observed_at: "2026-07-27",
  };
  const result = classifyListPagesLiveFixtures([capture(pageContentHtml)], {
    live_environment_blockers: [blocker],
  });
  assert.equal(result.schema, LISTPAGES_LIVE_FIXTURE_CLASSIFICATION_SCHEMA);
  assert.deepEqual(result.summary.capture_statuses, {captured: 1});
  assert.equal(result.summary.blocks, 2);
  assert.deepEqual(result.summary.block_class_counts, {"lp-a": 1, "lp-b": 1});
  assert.deepEqual(result.live_environment_blockers, [blocker]);
});

test("capture validation rejects stale HTML hashes", () => {
  const invalid = capture("<div id=\"page-content\"></div>");
  invalid.page_content_html += "changed";
  assert.throws(
    () => validateLiveFixtureCapture(invalid),
    /page content HTML hash does not match/u,
  );
});
