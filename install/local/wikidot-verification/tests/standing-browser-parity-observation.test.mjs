import assert from "node:assert/strict";
import test from "node:test";

import {
  captureDocumentObservation,
  observationArtifactName,
} from "../src/standing-browser-parity-observation.mjs";

test("immediate and settled browser artifacts have deterministic, distinct safe names", () => {
  const input = {
    label: "local",
    index: 0,
    url: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
  };
  const immediate = observationArtifactName({
    ...input,
    phase: "domcontentloaded-immediate",
  });
  const viewport = observationArtifactName({
    ...input,
    phase: "settled-viewport",
  });
  const fullPage = observationArtifactName({
    ...input,
    phase: "settled-full-page",
  });
  assert.match(
    immediate,
    /^standing-browser-local-00-[0-9a-f]{16}-domcontentloaded-immediate\.png$/u,
  );
  assert.notEqual(immediate, viewport);
  assert.notEqual(viewport, fullPage);
  assert.throws(
    () => observationArtifactName({ ...input, phase: "first-paint" }),
    /unsupported browser observation artifact phase/u,
  );
});

test("closed details descendants are excluded from rendered DOM and image counts", async () => {
  const box = { x: 0, y: 0, width: 100, height: 100 };
  const node = (localName, id) => ({
    localName,
    id,
    classList: [],
    children: [],
    parentElement: null,
    getBoundingClientRect: () => box,
    contains(candidate) {
      return this.children.some(
        (child) => child === candidate || child.contains(candidate),
      );
    },
    closest(selector) {
      for (let candidate = this; candidate; candidate = candidate.parentElement) {
        if (
          selector === "details:not([open])" &&
          candidate.localName === "details" &&
          !candidate.open
        ) {
          return candidate;
        }
      }
      return null;
    },
  });
  const append = (parent, ...children) => {
    parent.children.push(...children);
    for (const child of children) child.parentElement = parent;
  };
  const details = node("details", "closed");
  const summary = node("summary", "first-summary");
  const summaryChild = node("span", "summary-child");
  const secondSummary = node("summary", "second-summary");
  const hiddenImage = Object.assign(node("img", "hidden"), {
    complete: true,
    naturalWidth: 20,
    naturalHeight: 20,
    currentSrc: "https://example.test/hidden.png",
    src: "https://example.test/hidden.png",
  });
  append(summary, summaryChild);
  append(details, summary, secondSummary, hiddenImage);

  const outerDetails = node("details", "outer");
  const outerSummary = node("summary", "outer-summary");
  const innerDetails = node("details", "inner");
  const innerSummary = node("summary", "inner-summary");
  append(innerDetails, innerSummary);
  append(outerDetails, outerSummary, innerDetails);

  const observedNodes = [
    details,
    summary,
    summaryChild,
    secondSummary,
    hiddenImage,
    outerDetails,
    outerSummary,
    innerDetails,
    innerSummary,
  ];
  const root = { querySelectorAll: () => observedNodes };
  const fakeDocument = {
    images: [hiddenImage],
    documentElement: details,
    querySelector: (selector) => (selector === "#page-content" ? root : null),
    querySelectorAll: () => [],
  };
  const fakePage = {
    evaluate: async (callback, argument) => {
      const previousDocument = globalThis.document;
      const previousGetComputedStyle = globalThis.getComputedStyle;
      globalThis.document = fakeDocument;
      globalThis.getComputedStyle = () => ({
        display: "block",
        visibility: "visible",
        opacity: "1",
        getPropertyValue: () => "",
      });
      try {
        return callback(argument);
      } finally {
        globalThis.document = previousDocument;
        globalThis.getComputedStyle = previousGetComputedStyle;
      }
    },
  };
  const observation = await captureDocumentObservation(fakePage, {
    contract: {
      geometry_selectors: [],
      first_paint_geometry_selectors: [],
      presence_probes: [],
      first_paint_custom_properties: {},
    },
    phase: "settled",
    viewport: { width: 1366, height: 900 },
  });
  assert.deepEqual(observation.dom_signatures, [
    "details#closed",
    "details#outer",
    "span#summary-child",
    "summary#first-summary",
    "summary#outer-summary",
  ]);
  assert.equal(observation.rendered_images, 0);
});
