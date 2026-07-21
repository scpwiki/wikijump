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
  const summary = {
    localName: "summary",
    contains: () => false,
  };
  const details = {
    children: [summary],
    parentElement: null,
  };
  const visible = {
    localName: "p",
    id: "visible",
    classList: [],
    closest: () => null,
    getBoundingClientRect: () => box,
  };
  const hiddenImage = {
    localName: "img",
    id: "hidden",
    classList: [],
    complete: true,
    naturalWidth: 20,
    naturalHeight: 20,
    currentSrc: "https://example.test/hidden.png",
    src: "https://example.test/hidden.png",
    closest: () => details,
    getBoundingClientRect: () => box,
  };
  const root = { querySelectorAll: () => [visible, hiddenImage] };
  const fakeDocument = {
    images: [hiddenImage],
    documentElement: visible,
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
  assert.deepEqual(observation.dom_signatures, ["p#visible"]);
  assert.equal(observation.rendered_images, 0);
});
