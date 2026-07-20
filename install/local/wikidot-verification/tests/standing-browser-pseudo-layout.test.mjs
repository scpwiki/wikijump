import assert from "node:assert/strict";
import test from "node:test";

import {
  __test,
  applyCssBoxFallback,
  evaluatePseudoLayout,
} from "../src/standing-browser-pseudo-layout.mjs";

function snapshot() {
  const strings = [
    "",
    "before",
    "after",
    "hidden",
    "NFSI",
    "National Fog Safety Initiative",
  ];
  const styles = () => {
    const values = [];
    values[6] = 3;
    return values;
  };
  return {
    strings,
    documents: [
      {
        nodes: {
          parentIndex: [-1, 0, 1, 2, 2, 3, 3],
          backendNodeId: [1, 2, 3, 4, 5, 6, 7],
          pseudoType: { index: [4, 5, 6], value: [1, 1, 2] },
        },
        layout: {
          nodeIndex: [1, 2, 3, 4, 5, 6],
          bounds: [
            [0, 0, 200, 60],
            [10, 0, 100, 32],
            [40, 0, 100, 40],
            [10, 0, 20, 30],
            [40, 0, 50, 20],
            [40, 20, 100, 20],
          ],
          styles: [[], styles(), [], [], [], []],
          text: [0, 0, 0, 0, 4, 5],
        },
      },
    ],
  };
}

test("pseudo-layout capture retains generated text and clipping evidence", () => {
  const result = __test.captureFromSnapshot({
    snapshot: snapshot(),
    viewport: { width: 200, height: 100 },
    targets: [
      {
        id: "header_logo",
        pseudo: "::before",
        backend_node_ids: [3],
        visibility_container_backend_ids: [2],
      },
      {
        id: "header_title",
        pseudo: "::before",
        backend_node_ids: [4],
        visibility_container_backend_ids: [2],
      },
      {
        id: "header_subtitle",
        pseudo: "::after",
        backend_node_ids: [4],
        visibility_container_backend_ids: [2],
      },
    ],
  });
  assert.equal(result.header_title.descendant_text, "NFSI");
  assert.equal(
    result.header_subtitle.descendant_text,
    "National Fog Safety Initiative",
  );
  assert.equal(result.header_subtitle.visible_area_ratio, 0.42);
});

test("the pseudo-layout verdict rejects a clipped subtitle even when the node exists", () => {
  const result = evaluatePseudoLayout(
    {
      style: { content: '"National Fog Safety Initiative"' },
      pseudo_layout: {
        status: "captured",
        node_present: true,
        layout_present: true,
        painted_bounds: { x: 0, y: 20, width: 100, height: 20 },
        visible_area_ratio: 0.6,
        descendant_text: "National Fog Safety Initiative",
      },
    },
    {
      pseudo_layout: {
        require_generated_content: true,
        require_descendant_text: true,
        minimum_visible_area_ratio: 0.95,
      },
    },
  );
  assert.equal(result.status, "fail");
});

test("the CSS-box fallback retains clipping evidence for a background logo", () => {
  const layout = applyCssBoxFallback(
    {
      status: "captured",
      node_present: true,
      layout_present: false,
      painted_bounds: null,
      visible_bounds: null,
      visible_area_ratio: 0,
      clipping_ancestors: [{ bounds: { x: 0, y: 0, width: 100, height: 32 } }],
    },
    { x: 0, y: 0, width: 100, height: 32 },
    { width: "52px", height: "48px" },
  );
  assert.equal(layout.layout_kind, "css_box_fallback");
  assert.equal(layout.visible_area_ratio, 0.67);
});
