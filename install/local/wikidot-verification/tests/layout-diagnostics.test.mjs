import assert from "node:assert/strict";
import test from "node:test";

import {
  DEFAULT_COMPUTED_STYLE_WHITELIST,
  DEFAULT_SCP9506_DESCRIPTORS,
  collectElementDiagnostics,
  evaluateLayoutInvariants,
  layoutShiftSourceAttributionFromSnapshot,
  parseViewport,
} from "../src/layout-diagnostics.mjs";

test("parseViewport accepts width x height pairs", () => {
  assert.deepEqual(parseViewport("390x844"), {width: 390, height: 844});
  assert.deepEqual(parseViewport("1366X900"), {width: 1366, height: 900});
});

test("parseViewport rejects malformed values", () => {
  assert.throws(() => parseViewport("390"), /viewport must use WIDTHxHEIGHT/);
  assert.throws(() => parseViewport("0x844"), /positive integers/);
});

test("collectElementDiagnostics passes descriptors and style whitelist to the page", async () => {
  const calls = [];
  const page = {
    async evaluate(_fn, arg) {
      calls.push(arg);
      return [
        {
          name: "rate_widget",
          selector: ".page-rate-widget-box",
          found_count: 1,
          instances: [
            {
              index: 0,
              tag: "DIV",
              rect: {x: 10, y: 20, width: 120, height: 32, top: 20, right: 130, bottom: 52, left: 10},
              styles: {display: "block"},
              rendered: true,
              text: "rating: +371",
            },
          ],
        },
      ];
    },
  };

  const descriptors = [{name: "rate_widget", selector: ".page-rate-widget-box"}];
  const elements = await collectElementDiagnostics(page, descriptors, ["display"]);

  assert.deepEqual(calls, [{descriptors, computedStyles: ["display"], maxInstancesPerDescriptor: 20}]);
  assert.equal(elements[0].name, "rate_widget");
  assert.equal(elements[0].instances[0].rendered, true);
});

test("evaluateLayoutInvariants passes the planned scp-9506 healthy shape", () => {
  const diagnostics = {
    page: {
      status: 200,
      failed_requests: [],
      console_errors: [],
      document: {client_width: 1366, scroll_width: 1366},
    },
    elements: [
      element("main_content", true, ""),
      element("page_content", true, ""),
      element("rate_widget", true, "rating: +371"),
      element("ios_cache_issue_notification", false, ""),
      element("scp9506_local_file_images", true, "", {tag: "IMG", natural_width: 640, natural_height: 320}),
      element("collapsible_blocks", true, "More From This Author"),
    ],
  };

  const result = evaluateLayoutInvariants(diagnostics);

  assert.equal(result.summary.status, "pass");
  assert.equal(result.summary.failed, 0);
  assert.equal(result.anomalies.length, 0);
});

test("evaluateLayoutInvariants flags visible iOS cache overlay and missing rate value", () => {
  const diagnostics = {
    page: {
      status: 200,
      failed_requests: [],
      console_errors: [],
      document: {client_width: 390, scroll_width: 430},
    },
    elements: [
      element("main_content", true, ""),
      element("page_content", true, ""),
      element("rate_widget", true, "rating: 0"),
      element("ios_cache_issue_notification", true, "default theme failed to load"),
      element("scp9506_local_file_images", true, "", {tag: "IMG", natural_width: 640, natural_height: 320}),
      element("collapsible_blocks", true, "More From This AuthorMore From This Author"),
    ],
  };

  const result = evaluateLayoutInvariants(diagnostics);

  assert.equal(result.summary.status, "fail");
  assert.deepEqual(
    result.anomalies.map((item) => item.id),
    [
      "rate_widget_snapshot_value",
      "ios_cache_notification_hidden",
      "collapsible_labels_not_duplicated",
      "horizontal_overflow_recorded",
    ],
  );
});

test("default scp-9506 descriptors and computed styles include planned LD1 probes", () => {
  assert.ok(DEFAULT_SCP9506_DESCRIPTORS.some((item) => item.name === "ios_cache_issue_notification"));
  assert.ok(DEFAULT_SCP9506_DESCRIPTORS.some((item) => item.selector.includes("/local--files/scp-9506/")));
  assert.ok(DEFAULT_COMPUTED_STYLE_WHITELIST.includes("display"));
  assert.ok(DEFAULT_COMPUTED_STYLE_WHITELIST.includes("--logo"));
});

test("layoutShiftSourceAttributionFromSnapshot records compact image source hints", () => {
  const source = layoutShiftSourceAttributionFromSnapshot({
    tag: "IMG",
    id: "",
    classes: ["image", "crom-thumbnail", "extra-one", "extra-two"],
    alt: "NFSI",
    src: "https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png?cache=1",
    text: "",
  });

  assert.equal(source.tag, "img");
  assert.deepEqual(source.classes, ["image", "crom-thumbnail", "extra-one", "extra-two"]);
  assert.equal(source.alt, "NFSI");
  assert.equal(source.src, "https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png?cache=1");
  assert.equal(source.selector_hint, 'img.image.crom-thumbnail.extra-one[src*="/local--files/scp-9506/NFSI.png"]');
});

test("layoutShiftSourceAttributionFromSnapshot prefers ids and keeps text bounded", () => {
  const source = layoutShiftSourceAttributionFromSnapshot({
    tag: "DIV",
    id: "page-content",
    classes: ["content-panel", "unused"],
    role: "main",
    ariaLabel: "Article body",
    text: `The quick brown fox jumps over the lazy dog. ${"Repeated ".repeat(20)}`,
  });

  assert.equal(source.selector_hint, "div#page-content");
  assert.equal(source.role, "main");
  assert.equal(source.aria_label, "Article body");
  assert.ok(source.text.length <= 80);
  assert.ok(source.text.startsWith("The quick brown fox"));
});

test("layoutShiftSourceAttributionFromSnapshot does not turn javascript hrefs into selector hints", () => {
  const source = layoutShiftSourceAttributionFromSnapshot({
    tag: "A",
    href: "javascript:;",
    text: "COMMUNITY",
  });

  assert.equal(source.selector_hint, "a");
  assert.equal(source.href, "javascript:;");
  assert.equal(source.text, "COMMUNITY");
});

function element(name, rendered, text, overrides = {}) {
  return {
    name,
    selector: `[data-test="${name}"]`,
    found_count: 1,
    instances: [
      {
        index: 0,
        tag: overrides.tag ?? "DIV",
        rect: overrides.rect ?? {x: 0, y: 0, width: rendered ? 100 : 0, height: rendered ? 20 : 0, top: 0, right: 100, bottom: 20, left: 0},
        styles: {
          display: rendered ? "block" : "none",
          visibility: "visible",
          opacity: "1",
        },
        rendered,
        text,
        natural_width: overrides.natural_width,
        natural_height: overrides.natural_height,
      },
    ],
  };
}
