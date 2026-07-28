import fs from "node:fs/promises";
import path from "node:path";

import {
  sha256,
  validateWikidotReference,
} from "./syntax-differential.mjs";

export const LISTPAGES_PREVIEW_CLASSIFICATION_SCHEMA =
  "wikijump_listpages_compat.preview_classification.v1";

async function readJsonl(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  if (!text.trim()) return [];
  return text.trimEnd().split(/\r?\n/u).map((line) => JSON.parse(line));
}

function domHasClass(nodes, className) {
  for (const node of nodes ?? []) {
    const classes = node.attrs
      ?.find((attribute) => attribute.name === "class")
      ?.value.split(/\s+/u) ?? [];
    if (classes.includes(className) || domHasClass(node.children, className)) {
      return true;
    }
  }
  return false;
}

function localDom(row) {
  if (row.comparison?.checks?.dom_tree?.status === "mismatch") {
    return row.comparison.checks.dom_tree.local;
  }
  return null;
}

function templateVariables(source) {
  return [...source.matchAll(/%%[A-Za-z0-9_]+%%/gu)]
    .map((match) => match[0]);
}

function resolvesTemplateVariables(source, visibleText) {
  const variables = templateVariables(source);
  return variables.length > 0 &&
    variables.every((variable) => !visibleText.includes(variable));
}

function classifyMismatch(row, reference) {
  const source = reference.syntax_case.source;
  const liveText = row.live.visible_text;
  const localText = row.local?.visible_text ?? "";
  const liveHtml = reference.raw_html;
  const localNodes = localDom(row);
  const liveHasListPages =
    liveHtml.includes('class="list-pages-box"') ||
    liveHtml.includes('class="list-pages-item"') ||
    liveHtml.includes('class="pager"') ||
    resolvesTemplateVariables(source, liveText);
  const localHasListPages =
    domHasClass(localNodes, "list-pages-box") ||
    domHasClass(localNodes, "list-pages-item") ||
    domHasClass(localNodes, "pager") ||
    resolvesTemplateVariables(source, localText);
  const localPreservedModule =
    localText.includes("[[module ListPages") ||
    localText.includes("[[module\tListPages");

  const exactErrors = new Map([
    ["Invalid range argument.", ["invalid-range-error", "fix"]],
    ["Invalid pagetype attribute.", ["invalid-pagetype-error", "fix"]],
    ["Invalid rating argument.", ["invalid-rating-error", "fix"]],
    ["Invalid votes argument.", ["invalid-votes-error", "fix"]],
  ]);
  if (exactErrors.has(liveText)) {
    const [classification, disposition] = exactErrors.get(liveText);
    return {
      classification,
      disposition,
      rationale: "Live Wikidot emits a deterministic ListPages argument error.",
    };
  }
  if (/^Parent page .+ does not exist$/u.test(liveText)) {
    return {
      classification: "missing-parent-error",
      disposition: "fix",
      rationale: "Live Wikidot resolves the static parent and reports that it does not exist.",
    };
  }
  if (
    !source.includes("[[/module]]") &&
    /\[\[module\s+ListPages\b[^\n]*\]\]/iu.test(source) &&
    liveHasListPages &&
    localPreservedModule
  ) {
    return {
      classification: "unclosed-listpages-body-parser-gap",
      disposition: "investigate-parser",
      rationale: "Live executes a complete ListPages opening head without a closing module tag.",
    };
  }
  if (liveHasListPages && localPreservedModule) {
    return {
      classification: "live-parser-accepts-local-preserves",
      disposition: "minimize-parser",
      rationale: "Live executes the module while Wikijump leaves its source literal.",
    };
  }
  if (liveHasListPages && localHasListPages) {
    return {
      classification: "inconclusive-fixture-data-state",
      disposition: "replay-synchronized-fixture",
      rationale: "Both runtimes execute ListPages, but the live and local sites contain different pages.",
    };
  }
  if (liveHasListPages) {
    return {
      classification: "listpages-render-shape-divergence",
      disposition: "investigate-renderer",
      rationale: "Live emits a ListPages container while the local canonical DOM does not.",
    };
  }
  return {
    classification: "other-preview-divergence",
    disposition: "investigate",
    rationale: "The preview mismatch is not explained by a known argument, parser, or fixture-state class.",
  };
}

export async function classifyListPagesPreviewDifferential({
  verdictPath,
  referencesPath,
}) {
  const verdictText = await fs.readFile(verdictPath, "utf8");
  const verdict = JSON.parse(verdictText);
  const referencesText = await fs.readFile(referencesPath, "utf8");
  const references = (await readJsonl(referencesPath)).map(validateWikidotReference);
  const referencesById = new Map(
    references.map((reference) => [reference.syntax_case.case_id, reference]),
  );

  const cases = verdict.cases.map((row) => {
    const reference = referencesById.get(row.case_id);
    if (!reference) {
      throw new Error(`missing live reference for ${row.case_id}`);
    }
    const result = row.status === "match"
      ? {
          classification: "matched",
          disposition: "none",
          rationale: "Canonical DOM and visible text match.",
        }
      : row.status === "local-error"
        ? {
            classification: "local-preview-error",
            disposition: "fix-or-block",
            rationale: row.error,
          }
        : classifyMismatch(row, reference);
    return {
      schema: `${LISTPAGES_PREVIEW_CLASSIFICATION_SCHEMA}.case`,
      case_id: row.case_id,
      source: reference.syntax_case.source,
      source_sha256: reference.source_sha256,
      differential_status: row.status,
      live_html_sha256: reference.raw_html_sha256,
      local_html_sha256: row.local?.html_sha256 ?? null,
      ...result,
    };
  });

  if (cases.length !== references.length) {
    throw new Error(
      `verdict/reference case count differs: ${cases.length} != ${references.length}`,
    );
  }
  const counts = {};
  const dispositions = {};
  for (const row of cases) {
    counts[row.classification] = (counts[row.classification] ?? 0) + 1;
    dispositions[row.disposition] = (dispositions[row.disposition] ?? 0) + 1;
  }
  return {
    schema: LISTPAGES_PREVIEW_CLASSIFICATION_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      verdict_path: verdictPath,
      verdict_sha256: sha256(verdictText),
      references_path: referencesPath,
      references_sha256: sha256(referencesText),
    },
    cases,
    summary: {
      total: cases.length,
      classifications: counts,
      dispositions,
    },
  };
}

export async function writeListPagesPreviewClassification(
  classification,
  outputPath,
) {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(
    outputPath,
    `${JSON.stringify(classification, null, 2)}\n`,
    { mode: 0o600 },
  );
}
