const PSEUDO_STYLE_PROPERTIES = Object.freeze([
  "content",
  "background-image",
  "color",
  "display",
  "visibility",
  "opacity",
  "overflow",
  "overflow-x",
  "overflow-y",
  "clip",
  "clip-path",
  "contain",
  "font-family",
  "font-size",
  "font-weight",
  "line-height",
  "letter-spacing",
]);
const CLIPPING_OVERFLOWS = new Set(["hidden", "clip"]);

function rounded(value) {
  return Math.round(Number(value) * 100) / 100;
}

function stringValue(strings, value) {
  if (typeof value === "number") return strings?.[value] ?? "";
  return typeof value === "string" ? value : "";
}

function rareString(strings, rare, index) {
  const position = rare?.index?.indexOf(index) ?? -1;
  return position >= 0 ? stringValue(strings, rare.value?.[position]) : "";
}

function rectangle(bounds) {
  if (
    !Array.isArray(bounds) ||
    bounds.length < 4 ||
    !bounds.slice(0, 4).every(Number.isFinite)
  ) {
    return null;
  }
  const [x, y, width, height] = bounds;
  if (width < 0 || height < 0) return null;
  return {
    x: rounded(x),
    y: rounded(y),
    width: rounded(width),
    height: rounded(height),
  };
}

function area(bounds) {
  return bounds ? Math.max(0, bounds.width) * Math.max(0, bounds.height) : 0;
}

function union(bounds) {
  const valid = bounds.filter((candidate) => area(candidate) > 0);
  if (valid.length === 0) return null;
  const left = Math.min(...valid.map((candidate) => candidate.x));
  const top = Math.min(...valid.map((candidate) => candidate.y));
  const right = Math.max(
    ...valid.map((candidate) => candidate.x + candidate.width),
  );
  const bottom = Math.max(
    ...valid.map((candidate) => candidate.y + candidate.height),
  );
  return {
    x: rounded(left),
    y: rounded(top),
    width: rounded(right - left),
    height: rounded(bottom - top),
  };
}

function intersect(left, right) {
  if (!left || !right) return null;
  const x = Math.max(left.x, right.x);
  const y = Math.max(left.y, right.y);
  const rightEdge = Math.min(left.x + left.width, right.x + right.width);
  const bottomEdge = Math.min(left.y + left.height, right.y + right.height);
  return rightEdge > x && bottomEdge > y
    ? {
        x: rounded(x),
        y: rounded(y),
        width: rounded(rightEdge - x),
        height: rounded(bottomEdge - y),
      }
    : null;
}

function normalizedContent(value) {
  return String(value ?? "")
    .trim()
    .replace(/^(['"])|(['"])$/gu, "");
}

function cssPixels(value) {
  const match = /^(-?(?:\d+|\d*\.\d+))px$/u.exec(String(value ?? "").trim());
  return match && Number.isFinite(Number(match[1])) ? Number(match[1]) : null;
}

function isDescendant(nodes, candidate, ancestor) {
  for (
    let current = candidate;
    Number.isInteger(current) && current >= 0;
    current = nodes.parentIndex?.[current]
  ) {
    if (current === ancestor) return true;
  }
  return false;
}

function pseudoType(strings, nodes, index) {
  return rareString(strings, nodes.pseudoType, index)
    .replace(/^::/u, "")
    .toLowerCase();
}

function snapshotStyle(strings, layout, layoutIndex) {
  const values = layout.styles?.[layoutIndex] ?? [];
  return Object.fromEntries(
    PSEUDO_STYLE_PROPERTIES.map((property, index) => [
      property,
      stringValue(strings, values[index]),
    ]).filter(([, value]) => value !== ""),
  );
}

function clips(style) {
  const values = [style.overflow, style["overflow-x"], style["overflow-y"]].map(
    (value) =>
      String(value ?? "")
        .trim()
        .toLowerCase(),
  );
  return (
    values.some((value) => CLIPPING_OVERFLOWS.has(value)) ||
    (style.clip && style.clip !== "auto") ||
    (style["clip-path"] && style["clip-path"] !== "none") ||
    String(style.contain ?? "").includes("paint")
  );
}

function snapshotIndex(document) {
  const nodes = document.nodes ?? {};
  const layout = document.layout ?? {};
  const layoutByNode = new Map(
    (layout.nodeIndex ?? []).map((nodeIndex, layoutIndex) => [
      nodeIndex,
      layoutIndex,
    ]),
  );
  const nodeByBackend = new Map();
  for (const [nodeIndex, backendId] of (nodes.backendNodeId ?? []).entries()) {
    if (Number.isInteger(backendId) && backendId > 0) {
      nodeByBackend.set(backendId, nodeIndex);
    }
  }
  return { nodes, layout, layoutByNode, nodeByBackend };
}

function nodeBounds(index, nodeIndex) {
  const layoutIndex = index.layoutByNode.get(nodeIndex);
  return layoutIndex === undefined
    ? null
    : rectangle(index.layout.bounds?.[layoutIndex]);
}

function nodeStyle(strings, index, nodeIndex) {
  const layoutIndex = index.layoutByNode.get(nodeIndex);
  return layoutIndex === undefined
    ? {}
    : snapshotStyle(strings, index.layout, layoutIndex);
}

function directText(strings, nodes, nodeIndex) {
  return [
    stringValue(strings, nodes.nodeValue?.[nodeIndex]),
    rareString(strings, nodes.textValue, nodeIndex),
  ]
    .filter(Boolean)
    .join(" ")
    .replace(/\s+/gu, " ")
    .trim();
}

function layoutText(strings, index, nodeIndex) {
  const layoutIndex = index.layoutByNode.get(nodeIndex);
  return layoutIndex === undefined
    ? ""
    : stringValue(strings, index.layout.text?.[layoutIndex])
        .replace(/\s+/gu, " ")
        .trim();
}

function emptyPseudoLayout(status, error = null) {
  return {
    status,
    ...(error ? { error } : {}),
    node_present: false,
    layout_present: false,
    border_box: null,
    painted_bounds: null,
    visible_bounds: null,
    visible_area_ratio: 0,
    descendant_text: "",
    clipping_ancestors: [],
    computed_style: {},
  };
}

function captureFromSnapshot({ snapshot, targets, viewport }) {
  const document = snapshot?.documents?.[0];
  if (!document)
    throw new Error("DOMSnapshot did not include the main document");
  const strings = snapshot.strings ?? [];
  const index = snapshotIndex(document);
  const result = {};
  for (const target of targets) {
    const sourceNode =
      target.backend_node_ids.length === 1
        ? index.nodeByBackend.get(target.backend_node_ids[0])
        : undefined;
    const containerNode =
      target.visibility_container_backend_ids.length === 1
        ? index.nodeByBackend.get(target.visibility_container_backend_ids[0])
        : undefined;
    if (!Number.isInteger(sourceNode)) {
      result[target.id] = emptyPseudoLayout("source_not_found");
      continue;
    }
    const wantedPseudo = target.pseudo.replace(/^::/u, "").toLowerCase();
    const candidates = [];
    for (
      let nodeIndex = 0;
      nodeIndex < (index.nodes.parentIndex?.length ?? 0);
      nodeIndex += 1
    ) {
      if (
        pseudoType(strings, index.nodes, nodeIndex) === wantedPseudo &&
        index.nodes.parentIndex?.[nodeIndex] === sourceNode
      ) {
        candidates.push(nodeIndex);
      }
    }
    const pseudoNode = candidates.length === 1 ? candidates[0] : undefined;
    if (!Number.isInteger(pseudoNode)) {
      result[target.id] = emptyPseudoLayout(
        candidates.length === 0 ? "pseudo_not_found" : "pseudo_ambiguous",
      );
      continue;
    }
    const descendantNodes = [];
    for (
      let nodeIndex = 0;
      nodeIndex < (index.nodes.parentIndex?.length ?? 0);
      nodeIndex += 1
    ) {
      if (isDescendant(index.nodes, nodeIndex, pseudoNode)) {
        descendantNodes.push(nodeIndex);
      }
    }
    const boxes = descendantNodes
      .map((nodeIndex) => nodeBounds(index, nodeIndex))
      .filter(Boolean);
    const borderBox = nodeBounds(index, pseudoNode);
    const paintedBounds = union([borderBox, ...boxes]);
    const clippingAncestors = [];
    let visibleBounds = paintedBounds;
    const applyClip = (kind, nodeIndex, bounds, style = {}) => {
      if (!bounds) return;
      clippingAncestors.push({ kind, node_index: nodeIndex, bounds, style });
      visibleBounds = intersect(visibleBounds, bounds);
    };
    applyClip("viewport", null, {
      x: 0,
      y: 0,
      width: viewport.width,
      height: viewport.height,
    });
    if (Number.isInteger(containerNode)) {
      applyClip(
        "contract_container",
        containerNode,
        nodeBounds(index, containerNode),
      );
    }
    for (
      let ancestor = index.nodes.parentIndex?.[pseudoNode];
      Number.isInteger(ancestor) && ancestor >= 0;
      ancestor = index.nodes.parentIndex?.[ancestor]
    ) {
      const style = nodeStyle(strings, index, ancestor);
      if (clips(style)) {
        applyClip(
          "css_clipping_ancestor",
          ancestor,
          nodeBounds(index, ancestor),
          style,
        );
      }
    }
    const paintedArea = area(paintedBounds);
    const visibleArea = area(visibleBounds);
    result[target.id] = {
      status: "captured",
      node_present: true,
      layout_present: Boolean(borderBox || paintedBounds),
      border_box: borderBox,
      painted_bounds: paintedBounds,
      visible_bounds: visibleBounds,
      visible_area_ratio:
        paintedArea > 0 ? rounded(visibleArea / paintedArea) : 0,
      descendant_text: descendantNodes
        .map((nodeIndex) =>
          [
            layoutText(strings, index, nodeIndex),
            directText(strings, index.nodes, nodeIndex),
          ]
            .filter(Boolean)
            .join(" "),
        )
        .filter(Boolean)
        .join(" ")
        .replace(/\s+/gu, " ")
        .trim(),
      clipping_ancestors: clippingAncestors,
      computed_style: nodeStyle(strings, index, pseudoNode),
    };
  }
  return result;
}

async function backendNodeIds(client, rootNodeId, selector) {
  const { nodeIds = [] } = await client.send("DOM.querySelectorAll", {
    nodeId: rootNodeId,
    selector,
  });
  const descriptions = await Promise.all(
    nodeIds.map((nodeId) =>
      client.send("DOM.describeNode", { nodeId, depth: 0 }),
    ),
  );
  return descriptions
    .map(({ node }) => node?.backendNodeId)
    .filter((nodeId) => Number.isInteger(nodeId) && nodeId > 0);
}

function unavailable(probes, message) {
  return Object.fromEntries(
    probes
      .filter((probe) => probe.pseudo)
      .map((probe) => [probe.id, emptyPseudoLayout("capture_error", message)]),
  );
}

export async function capturePseudoLayouts(page, probes, viewport) {
  const pseudoProbes = probes.filter((probe) => probe.pseudo);
  if (pseudoProbes.length === 0) return {};
  let client;
  try {
    client = await page.context().newCDPSession(page);
    const { root } = await client.send("DOM.getDocument", {
      depth: 0,
      pierce: true,
    });
    const selectors = [
      ...new Set(
        pseudoProbes.flatMap((probe) =>
          [probe.selector, probe.visibility_container_selector].filter(Boolean),
        ),
      ),
    ];
    const idsBySelector = new Map(
      await Promise.all(
        selectors.map(async (selector) => [
          selector,
          await backendNodeIds(client, root.nodeId, selector),
        ]),
      ),
    );
    const targets = pseudoProbes.map((probe) => ({
      id: probe.id,
      pseudo: probe.pseudo,
      backend_node_ids: idsBySelector.get(probe.selector) ?? [],
      visibility_container_backend_ids: probe.visibility_container_selector
        ? (idsBySelector.get(probe.visibility_container_selector) ?? [])
        : [],
    }));
    const snapshot = await client.send("DOMSnapshot.captureSnapshot", {
      computedStyles: [...PSEUDO_STYLE_PROPERTIES],
      includeDOMRects: true,
      includePaintOrder: true,
    });
    return captureFromSnapshot({ snapshot, targets, viewport });
  } catch (error) {
    return unavailable(probes, error.message ?? String(error));
  } finally {
    await client?.detach().catch(() => {});
  }
}

export function evaluatePseudoLayout(probe, requirement) {
  const contract = requirement?.pseudo_layout;
  if (!contract) return { status: "not_applicable", checks: [] };
  const layout = probe?.pseudo_layout ?? {};
  const style = { ...(layout.computed_style ?? {}), ...(probe?.style ?? {}) };
  const checks = [
    {
      id: "capture",
      status: layout.status === "captured" ? "pass" : "fail",
      actual: layout.status ?? "missing",
    },
    {
      id: "node",
      status: layout.node_present === true ? "pass" : "fail",
      actual: layout.node_present === true,
    },
    {
      id: "layout",
      status:
        layout.layout_present === true && area(layout.painted_bounds) > 0
          ? "pass"
          : "fail",
      actual: layout.painted_bounds ?? null,
    },
    {
      id: "visible_area",
      status:
        Number(layout.visible_area_ratio) >=
        (contract.minimum_visible_area_ratio ?? 0.95)
          ? "pass"
          : "fail",
      actual: layout.visible_area_ratio ?? null,
      minimum: contract.minimum_visible_area_ratio ?? 0.95,
    },
  ];
  if (contract.require_background_image) {
    checks.push({
      id: "background_image",
      status:
        style["background-image"] && style["background-image"] !== "none"
          ? "pass"
          : "fail",
      actual: style["background-image"] ?? null,
    });
  }
  if (contract.require_generated_content) {
    checks.push({
      id: "generated_content",
      status:
        normalizedContent(style.content) !== "" &&
        normalizedContent(style.content) !== "none"
          ? "pass"
          : "fail",
      actual: style.content ?? null,
    });
  }
  if (contract.require_descendant_text) {
    checks.push({
      id: "descendant_text",
      status:
        String(layout.descendant_text ?? "").trim() !== "" ? "pass" : "fail",
      actual: layout.descendant_text ?? "",
    });
  }
  return {
    status: checks.every((check) => check.status === "pass") ? "pass" : "fail",
    checks,
  };
}

export function applyCssBoxFallback(layout, sourceRect, style) {
  if (
    !layout ||
    layout.status !== "captured" ||
    layout.painted_bounds ||
    !sourceRect
  ) {
    return layout;
  }
  const width = cssPixels(style?.width);
  const height = cssPixels(style?.height);
  if (!(width > 0 && height > 0)) return layout;
  const paintedBounds = {
    x: rounded(sourceRect.x),
    y: rounded(sourceRect.y),
    width: rounded(width),
    height: rounded(height),
  };
  let visibleBounds = paintedBounds;
  for (const clip of layout.clipping_ancestors ?? []) {
    visibleBounds = intersect(visibleBounds, clip.bounds);
  }
  const paintedArea = area(paintedBounds);
  return {
    ...layout,
    layout_present: true,
    layout_kind: "css_box_fallback",
    painted_bounds: paintedBounds,
    visible_bounds: visibleBounds,
    visible_area_ratio:
      paintedArea > 0 ? rounded(area(visibleBounds) / paintedArea) : 0,
  };
}

export function comparePseudoLayouts(
  localProbe,
  liveProbe,
  requirement,
  thresholds,
  { compareGeometry = false } = {},
) {
  const local = evaluatePseudoLayout(localProbe, requirement);
  const live = evaluatePseudoLayout(liveProbe, requirement);
  const localBounds = localProbe?.pseudo_layout?.painted_bounds ?? null;
  const liveBounds = liveProbe?.pseudo_layout?.painted_bounds ?? null;
  const ratioDelta = rounded(
    Math.abs(
      Number(localProbe?.pseudo_layout?.visible_area_ratio ?? 0) -
        Number(liveProbe?.pseudo_layout?.visible_area_ratio ?? 0),
    ),
  );
  const geometry =
    compareGeometry && localBounds && liveBounds
      ? Object.fromEntries(
          ["x", "y", "width", "height"].map((key) => [
            key,
            rounded(localBounds[key] - liveBounds[key]),
          ]),
        )
      : null;
  const geometryPasses =
    !compareGeometry ||
    (geometry &&
      Math.abs(geometry.x) <= thresholds.geometry_position_px &&
      Math.abs(geometry.y) <= thresholds.geometry_position_px &&
      Math.abs(geometry.width) <= thresholds.geometry_size_px &&
      Math.abs(geometry.height) <= thresholds.geometry_size_px);
  const ratioPasses = ratioDelta <= 0.05;
  return {
    local,
    live,
    ratio_delta: ratioDelta,
    geometry,
    status:
      local.status === "pass" &&
      live.status === "pass" &&
      geometryPasses &&
      ratioPasses
        ? "pass"
        : "fail",
  };
}

export const __test = Object.freeze({
  captureFromSnapshot,
  rectangle,
  union,
  intersect,
});
