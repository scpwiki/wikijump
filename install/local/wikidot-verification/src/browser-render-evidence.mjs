import fs from "node:fs/promises";
import path from "node:path";

import {
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
} from "./browser-render-target.mjs";

export {
  inventoryRows,
  isObject,
  rowLocalUrl,
  rowSourceUrl,
  rowsForShard,
  safePathSegment,
  selectInventoryRows,
} from "./browser-render-target.mjs";

export function compactVisibleText(value) {
  return String(value ?? "").replace(/\s+/g, " ").trim();
}

export async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

export function buildEvidenceRecord({row, source, local, sourceArtifact, localArtifact, sourceScreenshot, localScreenshot, localUrlField}) {
  const fixtureId = row.fixture_id;
  if (!fixtureId) {
    throw new Error("inventory row is missing fixture_id");
  }

  return {
    schema: "wikijump_full_parity.browser_rendering_record.v1",
    evidence_type: "browser_rendering",
    fixture_id: fixtureId,
    family: row.family ?? null,
    slug: row.slug ?? null,
    source_url: rowSourceUrl(row),
    local_url: rowLocalUrl(row, localUrlField),
    source_browser_artifact: sourceArtifact,
    local_browser_artifact: localArtifact,
    source_screenshot_artifact: sourceScreenshot ?? null,
    local_screenshot_artifact: localScreenshot ?? null,
    source_visible_text: compactVisibleText(source?.visibleText),
    local_visible_text: compactVisibleText(local?.visibleText),
    source_status: source?.status ?? null,
    local_status: local?.status ?? null,
    source_final_url: source?.finalUrl ?? null,
    local_final_url: local?.finalUrl ?? null,
    source_console_errors: source?.consoleErrors ?? [],
    local_console_errors: local?.consoleErrors ?? [],
    source_failed_requests: source?.failedRequests ?? [],
    local_failed_requests: local?.failedRequests ?? [],
    capture_errors: [
      ...(source?.error ? [{side: "source", message: source.error}] : []),
      ...(local?.error ? [{side: "local", message: local.error}] : []),
    ],
  };
}

export async function writeEvidenceArtifacts({outputDir, row, source, local, screenshot}) {
  const rowDir = path.join(outputDir, safePathSegment(row.fixture_id));
  await fs.mkdir(rowDir, {recursive: true});

  const sourceDom = path.join(rowDir, "live.dom.html");
  const localDom = path.join(rowDir, "local.dom.html");
  await fs.writeFile(sourceDom, source?.html ?? "", "utf8");
  await fs.writeFile(localDom, local?.html ?? "", "utf8");

  return {
    sourceArtifact: sourceDom,
    localArtifact: localDom,
    sourceScreenshot: screenshot ? path.join(rowDir, "live.png") : null,
    localScreenshot: screenshot ? path.join(rowDir, "local.png") : null,
  };
}
