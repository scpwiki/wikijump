#!/usr/bin/env node

import {
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  validateWikidotImplementationLedger,
} from "./lib/wikidot-implementation-ledger.mjs";
import {
  parseWikidotLiveEvidenceRows,
  resolveWikidotLiveEvidenceFormat,
  verifiedExternalEvidenceCaseIds,
  verifiedNativeThemeCaseIds,
} from "./lib/wikidot-live-evidence.mjs";
import { escapeMarkdownTableCell } from "./lib/markdown.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..");

// Per-run memoization of evidence bytes. The same evidence file is referenced
// by many observations, contracts, and aliases; reading and re-hashing it once
// per reference is avoidable I/O. Scoped to this process only, and every
// reference still re-verifies its own expected SHA-256 against the bytes.
const evidenceFileBytes = new Map();
function readEvidenceFile(path) {
  let bytes = evidenceFileBytes.get(path);
  if (bytes === undefined) {
    bytes = readFileSync(path);
    evidenceFileBytes.set(path, bytes);
  }
  return bytes;
}
const outputRoot = join(repositoryRoot, "docs", "wikidot-specifications");
const specificationsRoot = join(outputRoot, "specifications");
const liveObservationsSourcePath = join(
  scriptDirectory,
  "data",
  "wikidot-live-observations.json",
);
const implementationLedgerSourcePath = join(
  scriptDirectory,
  "data",
  "wikidot-implementation-ledger.json",
);
const detailedContractsSourcePath = join(
  scriptDirectory,
  "data",
  "wikidot-detailed-feature-contracts.json",
);
const detailedSpecEvidenceSourcePath = join(
  scriptDirectory,
  "data",
  "wikidot-detailed-spec-evidence-20260816.json",
);
const detailedContractsReferentSourcePath = join(
  scriptDirectory,
  "data",
  "referent-table-detailed-feature-contracts.md",
);
const corpusRoot = resolve(
  process.env.WIKIDOT_DOCUMENTATION_CORPUS ??
    "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages",
);
const displayedCorpusRoot =
  "~/src/Rokurolize/scp-wiki-translation/corpus/www/pages";

const checkOnly = process.argv.includes("--check");
const schemaVersion = 1;
const generatedDate = "2026-07-28";
const detailedContractAxes = Object.freeze({
  P1: "invocation grammar and scalar interpretation",
  P2: "parser stage, nesting, and composition",
  P3: "lifecycle, persistence, import, and round trips",
  P4: "actors, permissions, visibility, and privacy",
  P5: "selection, ordering, counting, and pagination",
  P6: "HTTP, API, URL, Ajax, feed, and navigation contracts",
  P7: "DOM, CSS, resources, interaction, and geometry",
  P8: "temporal behavior, failure atomicity, limits, and resource bounds",
});

function invariant(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function titleCase(value) {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/[_-]+/g, " ")
    .replace(/\b\w/g, (character) => character.toUpperCase());
}

function corpusPath(fullname) {
  return `${displayedCorpusRoot}/${fullname}/source.wikidot.txt`;
}

function loadPages() {
  const pages = new Map();

  for (const entry of readdirSync(corpusRoot, { withFileTypes: true }).sort(
    (left, right) => left.name.localeCompare(right.name),
  )) {
    if (!entry.isDirectory()) {
      continue;
    }

    const directory = join(corpusRoot, entry.name);
    const sourcePath = join(directory, "source.wikidot.txt");
    const metadataPath = join(directory, "meta.json");
    const source = readFileSync(sourcePath, "utf8");
    const metadata = JSON.parse(readFileSync(metadataPath, "utf8"));
    const lines = source.split(/\r?\n/);
    if (lines.at(-1) === "") {
      lines.pop();
    }

    pages.set(entry.name, {
      fullname: entry.name,
      directory,
      source,
      lines,
      lineCount: lines.length,
      sha256: sha256(source),
      metadata,
    });
  }

  return pages;
}

const pages = loadPages();
invariant(
  pages.size === 1806,
  `Expected 1806 corpus pages, found ${pages.size}`,
);

const liveObservations = JSON.parse(
  readFileSync(liveObservationsSourcePath, "utf8"),
);
invariant(
  liveObservations.schema === "wikijump.wikidot_live_observations.v1",
  "Unexpected live observation schema",
);
invariant(
  Array.isArray(liveObservations.observations),
  "Live observations must be an array",
);
const specSupplements = JSON.parse(
  readFileSync(join(repositoryRoot, "scripts/data/wikidot-spec-supplements.json"), "utf8"),
);
invariant(
  specSupplements.schema === "wikijump.wikidot_spec_supplements.v1" &&
    Array.isArray(specSupplements.supplements),
  "Unexpected spec supplement schema",
);
for (const supplement of specSupplements.supplements) {
  invariant(
    typeof supplement.feature_id === "string" &&
      typeof supplement.title === "string" &&
      supplement.title !== "" &&
      typeof supplement.markdown === "string" &&
      supplement.markdown.trim() !== "",
    `Invalid spec supplement: ${supplement.title}`,
  );
}
const liveObservationIds = new Set();
for (const observation of liveObservations.observations) {
  invariant(
    /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(observation.id),
    `Invalid live observation id: ${observation.id}`,
  );
  invariant(
    !liveObservationIds.has(observation.id),
    `Duplicate live observation id: ${observation.id}`,
  );
  liveObservationIds.add(observation.id);
  invariant(
    Array.isArray(observation.feature_ids) &&
      observation.feature_ids.length > 0,
    `Live observation has no feature IDs: ${observation.id}`,
  );
  invariant(
    Array.isArray(observation.normative_behavior) &&
      observation.normative_behavior.length > 0,
    `Live observation has no normative behavior: ${observation.id}`,
  );
  invariant(
    Array.isArray(observation.evidence) && observation.evidence.length > 0,
    `Live observation has no evidence: ${observation.id}`,
  );
  for (const rawCapture of observation.raw_captures ?? []) {
    invariant(
      typeof rawCapture.path === "string" && rawCapture.path.startsWith("/"),
      `Live raw capture must use an absolute path for ${observation.id}`,
    );
    invariant(
      sha256(readEvidenceFile(rawCapture.path)) === rawCapture.sha256,
      `Live raw capture hash drifted for ${observation.id}: ${rawCapture.path}`,
    );
  }
  for (const evidence of observation.evidence) {
    const evidencePath = resolve(repositoryRoot, evidence.path);
    const rawEvidence = readEvidenceFile(evidencePath).toString("utf8");
    invariant(
      sha256(rawEvidence) === evidence.sha256,
      `Live evidence hash drifted for ${observation.id}: ${evidence.path}`,
    );
    const evidenceRows = parseWikidotLiveEvidenceRows(
      rawEvidence,
      resolveWikidotLiveEvidenceFormat(evidence),
    );
    const externalCaseIds = new Set();
    for (const row of evidenceRows) {
      for (const caseId of verifiedExternalEvidenceCaseIds(row, evidenceFileBytes)) {
        externalCaseIds.add(caseId);
      }
    }
    const capturedCaseIds = new Set();
    for (const row of evidenceRows) {
      for (const caseId of verifiedNativeThemeCaseIds(row, dirname(evidencePath), evidenceFileBytes)) capturedCaseIds.add(caseId);
      if (
        row.schema === "wikijump_syntax_differential.saved_page_probe.v1" &&
        typeof row.fullname === "string"
      ) {
        capturedCaseIds.add(row.fullname);
      }
      if (row.case_id) {
        capturedCaseIds.add(row.case_id);
      }
      if (row.syntax_case?.case_id) {
        capturedCaseIds.add(row.syntax_case.case_id);
      }
      if (row.case?.case_id) {
        capturedCaseIds.add(row.case.case_id);
      }
      for (const capture of row.captures ?? []) {
        if (capture.case_id) {
          capturedCaseIds.add(capture.case_id);
        }
      }
      if (Array.isArray(row.case_ids)) {
        for (const caseId of row.case_ids) {
          capturedCaseIds.add(caseId);
        }
      }
      if (Array.isArray(row.cases)) {
        for (const capture of row.cases) {
          if (capture.case_id) {
            capturedCaseIds.add(capture.case_id);
          }
        }
      } else if (row.cases !== null && typeof row.cases === "object") {
        for (const [caseId, capture] of Object.entries(row.cases)) {
          capturedCaseIds.add(caseId);
          if (capture?.case_id) {
            capturedCaseIds.add(capture.case_id);
          }
        }
      }
      for (const observation of row.observations ?? []) {
        if (observation.case_id) {
          capturedCaseIds.add(observation.case_id);
        }
      }
      for (const fieldRun of row.field_runs ?? []) {
        for (const control of fieldRun.controls ?? []) {
          if (control.case_id) {
            capturedCaseIds.add(control.case_id);
          }
        }
      }
      for (const rule of row.general_rules ?? []) {
        for (const caseId of rule.case_ids ?? []) {
          invariant(
            externalCaseIds.has(caseId),
            `Summary live evidence case ${caseId} is not backed by a verified raw capture`,
          );
          capturedCaseIds.add(caseId);
        }
      }
    }
    for (const caseId of evidence.case_ids) {
      invariant(
        capturedCaseIds.has(caseId),
        `Live evidence case ${caseId} is missing from ${evidence.path}`,
      );
    }
  }
}

const detailedContracts = JSON.parse(
  readFileSync(detailedContractsSourcePath, "utf8"),
);
const detailedSpecEvidence = JSON.parse(
  readFileSync(detailedSpecEvidenceSourcePath, "utf8"),
);
const detailedContractsReferent = readFileSync(
  detailedContractsReferentSourcePath,
  "utf8",
);

const detailedEvidenceAliases = Object.freeze({
  "current-www-source": detailedSpecEvidence.captures?.current_www_sources,
  "invocation-only-module-pagepreview":
    detailedSpecEvidence.captures?.invocation_only_module_pagepreview,
  "special-page-module-summary":
    detailedSpecEvidence.captures?.special_page_module_summary,
  "syntax-pagepreview": detailedSpecEvidence.captures?.syntax_pagepreview,
  "authenticated-structure":
    detailedSpecEvidence.captures?.authenticated_structure,
  "account-upgrade-nonpro":
    detailedSpecEvidence.captures?.account_upgrade_nonpro,
  "data-form-public-demos":
    detailedSpecEvidence.captures?.data_form_public_demos,
  "expressions-live-probes":
    detailedSpecEvidence.captures?.expressions_live_probes,
  "expressions-length-probes":
    detailedSpecEvidence.captures?.expressions_length_probes,
  "expressions-length-boundary-probes":
    detailedSpecEvidence.captures?.expressions_length_boundary_probes,
  "data-form-create-edit":
    detailedSpecEvidence.retained_repository_evidence?.data_form_create_edit,
  "data-form-date-pagepath":
    detailedSpecEvidence.retained_repository_evidence?.data_form_date_pagepath,
  "data-form-pagepath-control":
    detailedSpecEvidence.retained_repository_evidence?.data_form_pagepath_control,
  "data-form-pagepath-create-new":
    detailedSpecEvidence.retained_repository_evidence?.data_form_pagepath_create_new,
  "data-form-pagepath-root-bootstrap":
    detailedSpecEvidence.retained_repository_evidence?.data_form_pagepath_root_bootstrap,
  "data-form-pagepath-backlinks":
    detailedSpecEvidence.retained_repository_evidence?.data_form_pagepath_backlinks,
  "data-form-file-field":
    detailedSpecEvidence.retained_repository_evidence?.data_form_file_field,
  "data-form-images-links-youtube":
    detailedSpecEvidence.retained_repository_evidence?.data_form_images_links_youtube,
  "data-form-youtube":
    detailedSpecEvidence.retained_repository_evidence?.data_form_youtube,
  "data-form-output-css":
    detailedSpecEvidence.retained_repository_evidence?.data_form_output_css,
  "userinfo-targets":
    detailedSpecEvidence.retained_repository_evidence?.userinfo_targets,
  "category-template-lifecycle":
    detailedSpecEvidence.retained_repository_evidence?.category_template_lifecycle,
  "canonical-live-observations":
    detailedSpecEvidence.retained_repository_evidence?.canonical_live_observations,
});

function page(fullname) {
  const value = pages.get(fullname);
  invariant(value, `Corpus page is missing: ${fullname}`);
  return value;
}

function lineReference(
  fullname,
  startLine = 1,
  endLine = undefined,
  role = "canonical",
) {
  const sourcePage = page(fullname);
  const actualEnd =
    endLine === undefined ? Math.max(sourcePage.lineCount, 1) : endLine;
  invariant(startLine >= 1, `Invalid start line for ${fullname}: ${startLine}`);
  invariant(
    sourcePage.lineCount === 0 || actualEnd <= sourcePage.lineCount,
    `Invalid end line for ${fullname}: ${actualEnd}`,
  );
  invariant(
    sourcePage.lineCount === 0 || startLine <= actualEnd,
    `Invalid source range for ${fullname}: ${startLine}-${actualEnd}`,
  );

  return {
    fullname,
    start_line: startLine,
    end_line: actualEnd,
    role,
  };
}

function headings(fullname) {
  return page(fullname)
    .lines.map((line, index) => {
      const match = line.match(/^(\+{1,6})\s+(.+?)\s*$/);
      if (!match) {
        return null;
      }
      return {
        level: match[1].length,
        text: match[2]
          .replace(/\[\[#.*?\]\]/g, "")
          .replace(/\{\{@@/g, "")
          .replace(/@@\}\}/g, "")
          .trim(),
        line: index + 1,
      };
    })
    .filter(Boolean);
}

function sectionReference(fullname, headingText, role = "canonical") {
  const pageHeadings = headings(fullname);
  const targetIndex = pageHeadings.findIndex(
    (heading) => heading.text.toLowerCase() === headingText.toLowerCase(),
  );
  invariant(
    targetIndex >= 0,
    `Heading "${headingText}" not found in ${fullname}`,
  );
  const target = pageHeadings[targetIndex];
  const following = pageHeadings
    .slice(targetIndex + 1)
    .find((heading) => heading.level <= target.level);
  return lineReference(
    fullname,
    target.line,
    following ? following.line - 1 : Math.max(page(fullname).lineCount, 1),
    role,
  );
}

const features = [];
const featureIds = new Set();

function addFeature({
  id,
  title,
  category,
  summary,
  sources,
  seams,
  documentationStatus = "documented",
  implementationNotes = [],
  relatedFeatures = [],
}) {
  invariant(/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id), `Invalid feature id: ${id}`);
  invariant(!featureIds.has(id), `Duplicate feature id: ${id}`);
  invariant(sources.length > 0, `Feature has no source: ${id}`);
  for (const source of sources) {
    page(source.fullname);
  }
  featureIds.add(id);
  features.push({
    id,
    title,
    category,
    summary,
    documentation_status: documentationStatus,
    sources,
    suggested_tdd_seams: seams,
    implementation_notes: implementationNotes,
    related_features: relatedFeatures,
  });
}

const syntaxSeams = [
  "FTML public parse/render interface using Wikidot layout",
  "Rendered HTML/DOM at the saved-page boundary for context-dependent forms",
];
const moduleSeams = [
  "Saved-page or preview rendering through Deepwell's public page-view interface",
  "Framerail HTTP/browser boundary when the module is interactive or URL-driven",
];
const dataFormSeams = [
  "Data-form template parsing and saved page rendering",
  "Public create/edit/view flow and ListPages query behavior where documented",
];
const platformSeams = [
  "Public HTTP route and browser-visible UI",
  "Public service/API boundary for persistent state and permissions",
];
const apiSeams = [
  "Published Wikidot API method boundary",
  "Public persistence/query behavior reached through that method",
];

const syntaxPages = [...pages.keys()]
  .filter(
    (fullname) =>
      fullname.startsWith("doc-wiki-syntax:") &&
      !["doc-wiki-syntax:start", "doc-wiki-syntax:_template"].includes(
        fullname,
      ),
  )
  .sort();

for (const fullname of syntaxPages) {
  const sourcePage = page(fullname);
  const slug = fullname.slice("doc-wiki-syntax:".length);
  const supportingSources = [];
  if (slug === "embedding" || slug === "embedding-code") {
    supportingSources.push(
      lineReference("doc:embedding", 1, undefined, "supporting"),
    );
  }
  if (slug === "horizontal-rules") {
    supportingSources.push(
      lineReference("doc:quick-reference", 51, 51, "supporting"),
    );
  }
  addFeature({
    id: `syntax-${slugify(slug)}`,
    title: `${sourcePage.metadata.title} syntax`,
    category: "wiki-syntax",
    summary: `Parse and render Wikidot's documented ${sourcePage.metadata.title.toLowerCase()} syntax, including every documented form, option, output rule, and limitation.`,
    sources: [lineReference(fullname), ...supportingSources],
    seams: syntaxSeams,
    documentationStatus:
      sourcePage.lineCount === 0 ? "partially-documented" : "documented",
    implementationNotes:
      slug === "foldable-list" ? [
  "Frozen native `WIKIDOT.page.fixers.fixFoldableMenus` and `_foldableMenuToggle` establish browser behavior independently of the initial PagePreview DOM. The retained script object is SHA-256 `49bcabdb14d446bb93d261f9c967bedfa7cdf7b214807a8180974cbd00b21f11` in `install/local/theme-lab/ports/authority-evidence/replay/objects/49/`.",
  "The fixer hides each nested UL, remembers its authored inline display, adds `folded` to its nearest LI before the container boundary, and wraps that LI's first child in an anchor when necessary. Links to the current page unfold their ancestor list items.",
  "A container owns its click listener; the nearest authored LI owns `folded`/`unfolded` state. A control-only container can therefore operate on a surrounding LI. Real navigation links retain their navigation behavior. Fold controls toggle the first descendant UL and restore its remembered display when opened.",
  "These controls do not change the URL hash, create history entries, or send page mutations. Nested containers retain their native event bubbling behavior.",
  "The frozen Sigma-10 credit controls provide an independent settled-state browser proof: `install/local/theme-lab/ports/authority-evidence/native-sigma10-fold-controls-settled-20261002/receipt.json`. Trusted clicks cover open, nested otherwise, return, and close. The ordinary nested-list and replacement-article cases are exercised by `framerail/tests/wikidot-foldable-lists.spec.ts` in the normal product regression gate.",
  "Framerail owns this browser action; FTML continues to render the authored initial tree. Newly rendered article content must receive the same initialization, and obsolete listeners must be removed when its owner is destroyed."
] : sourcePage.lineCount === 0
        ? [
            "The canonical page has an empty source. The supporting quick-reference evidence is the complete documented contract in this snapshot.",
          ]
        : [],
  });
}

const modulePages = [...pages.keys()]
  .filter(
    (fullname) =>
      fullname.startsWith("doc-modules:") && fullname !== "doc-modules:start",
  )
  .sort();

const documentedModuleNames = new Map();

function moduleNamesForPage(fullname) {
  const basename = fullname
    .slice("doc-modules:".length)
    .replace(/-module$/, "");
  const special = {
    adsenseunit: ["AdSenseUnit"],
    nextpreviouspage: ["NextPage", "PreviousPage"],
  };
  if (special[basename]) {
    return special[basename];
  }
  const title = page(fullname).metadata.title.replace(/\s+Module$/i, "");
  return [title.replace(/\s+/g, "")];
}

for (const fullname of modulePages) {
  const sourcePage = page(fullname);
  const slug = fullname.slice("doc-modules:".length).replace(/-module$/, "");
  const id = `module-${slugify(slug)}`;
  const moduleNames = moduleNamesForPage(fullname);
  for (const moduleName of moduleNames) {
    documentedModuleNames.set(moduleName.toLowerCase(), id);
  }
  const sources = [lineReference(fullname)];
  if (fullname === "doc-modules:listpages-module") {
    sources.push(
      lineReference(
        "doc-include:note-template-in-modules",
        1,
        undefined,
        "included",
      ),
      lineReference("doc-include:page-selection", 1, undefined, "included"),
      lineReference(
        "doc-include:listpages-module-prev",
        1,
        undefined,
        "legacy",
      ),
    );
  }
  if (fullname === "doc-modules:countpages-module") {
    sources.push(
      lineReference("doc-include:page-selection", 1, undefined, "included"),
    );
  }
  addFeature({
    id,
    title: sourcePage.metadata.title,
    category: "module",
    summary: `Implement the ${moduleNames.map((name) => `\`${name}\``).join(" and ")} module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.`,
    sources,
    seams: moduleSeams,
    documentationStatus:
      sourcePage.lineCount === 0 ? "partially-documented" : "documented",
    implementationNotes: [
      "Module names and attribute names are compatibility-sensitive and must not be modernized.",
      "Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.",
      ...(fullname === "doc-modules:themepreviewer-module" ? ["Observation theme-previewer-blank-saved-page-and-preview-20261003 binds blank saved-page theme selection separately from empty PagePreview output. Its resolved executable include content owns persisted presentation state; code and comments remain inert. Unknown heads and values remain outside that observation."] : []),
      ...(fullname === "doc-modules:listpages-module"
        ? [
            "Wikijump runtime invariant (not a claim about live Wikidot): one root-and-nested render admits at most 512 ListPages modules, 2 MiB of aggregate matched module source, 256 KiB per template body, and 16 MiB of actual generated wikitext. A nested expansion pass additionally evaluates at most 64 modules before using the existing controlled unsupported-module diagnostic.",
            "Generated wrapper and item markup, sections, substituted rows, feed and pager markup, and post-wrapper paragraph repair consume one shared counter before append. Rows are atomic; a rejected module remains literal or uses an already evidenced controlled fallback, is never partially emitted, and must not broaden its query.",
            "Content-backed modules, including current-page-only modules, share a three-module and 100-row work budget before revision loading or nested include expansion.",
            "Random ListPages and CountPages examine at most 5,000 raw rows and filter the sample through anonymous view permissions. At the cap, ListPages returns the visible sample and CountPages returns its visible sampled count rather than a permission-dependent literal fallback; the sampled count is intentionally not exact.",
            "Regenerate `install/local/wikidot-verification/artifacts/listpages-runtime-budget-envelope.json` after a corpus refresh. Its deterministic check covers all 23,893 preserved invocation records and currently measures maxima of 273 modules per page, 112,646 aggregate matched-source bytes, 173,697 aggregate template-body bytes, 133,680 bytes for one template, and 13,964,000 estimated first-page authored-template bytes.",
          ]
        : []),
    ],
  });
}

documentedModuleNames.set("simpletodo", "module-simpletodo");
addFeature({
  id: "module-simpletodo",
  title: "SimpleToDo Module",
  category: "module",
  summary:
    "Implement Wikidot's deprecated SimpleToDo list module, including task mutation, attributes, permissions, and rendered controls.",
  sources: [lineReference("doc:simpletodo-module")],
  seams: moduleSeams,
  documentationStatus: "documented-deprecated",
});

const dataFormPages = [...pages.keys()]
  .filter(
    (fullname) =>
      fullname.startsWith("doc-data-forms:") &&
      !["doc-data-forms:thanks", "doc-data-forms:reference"].includes(fullname),
  )
  .sort();

for (const fullname of dataFormPages) {
  const sourcePage = page(fullname);
  const slug = fullname.slice("doc-data-forms:".length);
  const id =
    slug === "start" ? "data-forms-overview" : `data-forms-${slugify(slug)}`;
  addFeature({
    id,
    title: sourcePage.metadata.title,
    category: "data-forms",
    summary:
      slug === "start"
        ? "Support structured page data defined by category templates and exposed through Wikidot create, edit, display, and query flows."
        : `Implement the documented data-form capability “${sourcePage.metadata.title}”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.`,
    sources: [lineReference(fullname)],
    seams: dataFormSeams,
    documentationStatus:
      sourcePage.lineCount === 0 ? "partially-documented" : "documented",
    implementationNotes:
      sourcePage.lineCount === 0
        ? [
            "This snapshot names the feature but provides no body text. Do not invent deletion semantics without live-oracle evidence.",
          ]
        : [],
  });
}

const apiHeadings = headings("doc:api").filter(
  (heading) => heading.level === 2,
);
const firstApiHeading = apiHeadings[0];
addFeature({
  id: "api-overview",
  title: "Wikidot API overview",
  category: "api",
  summary:
    "Expose the documented Wikidot API authentication model, endpoint conventions, request rules, response conventions, and method namespace.",
  sources: [lineReference("doc:api", 1, firstApiHeading.line - 1)],
  seams: apiSeams,
});

for (const heading of apiHeadings) {
  const method = heading.text.trim();
  const isDeleted = method.toLowerCase() === "deleted methods";
  addFeature({
    id: isDeleted ? "api-deleted-methods" : `api-${slugify(method)}`,
    title: isDeleted ? "Removed Wikidot API methods" : `Wikidot API: ${method}`,
    category: "api",
    summary: isDeleted
      ? "Reject or omit API methods that the documentation explicitly records as deleted."
      : `Implement the \`${method}\` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.`,
    sources: [sectionReference("doc:api", method)],
    seams: apiSeams,
    documentationStatus: isDeleted ? "documented-negative" : "documented",
  });
}

const standaloneFeatures = [
  {
    id: "expressions",
    title: "Expressions",
    page: "doc:expressions",
    summary:
      "Evaluate Wikidot expressions with the documented grammar, operators, variables, coercions, and error behavior.",
    seams: syntaxSeams,
  },
  {
    id: "page-templates",
    title: "Category page templates",
    page: "doc:templates",
    summary:
      "Apply category `_template` pages, content splitting, variables, default content, hidden pages, and missing-page templates exactly as documented.",
    seams: [...syntaxSeams, ...platformSeams],
  },
  {
    id: "search-language",
    title: "Search query language",
    page: "doc:searching",
    summary:
      "Implement Wikidot's basic, filtered, global, and tag-oriented search behavior and query syntax.",
    seams: moduleSeams,
  },
  {
    id: "karma",
    title: "User karma",
    page: "doc:karma",
    summary:
      "Represent and display Wikidot user karma according to the documented visibility, progression, benefits, and anti-abuse behavior.",
    seams: platformSeams,
  },
  {
    id: "user-roles",
    title: "Wikidot users and site roles",
    page: "doc:users",
    summary:
      "Distinguish anonymous users, registered users, site members, moderators, administrators, and superusers with the documented status relationships.",
    seams: platformSeams,
  },
  {
    id: "advertising",
    title: "Site advertising",
    page: "doc:advertising",
    summary:
      "Apply Wikidot's documented advertising placement and account/site eligibility behavior.",
    seams: platformSeams,
  },
  {
    id: "thumbnails",
    title: "Page and site thumbnails",
    page: "doc:thumbnails",
    summary:
      "Generate and serve the documented thumbnail URL forms and size variants.",
    seams: platformSeams,
  },
];

for (const descriptor of standaloneFeatures) {
  addFeature({
    id: descriptor.id,
    title: descriptor.title,
    category: "platform",
    summary: descriptor.summary,
    sources: [lineReference(descriptor.page)],
    seams: descriptor.seams,
  });
}

const siteStructureSections = [
  ["Sites", "site-identity", "Sites and site identity"],
  ["Content pages", "content-pages", "Content pages"],
  ["Direct links between pages", "page-links", "Direct page links"],
  ["Page inclusions", "page-inclusions", "Page inclusion relationships"],
  [
    "Categories (namespaces)",
    "page-categories",
    "Page categories and namespaces",
  ],
  ["Tags", "page-tags", "Page tags"],
  ["Parent pages", "page-parent-relations", "Parent-page relations"],
  ["Forum", "forums-overview", "Site forums"],
  ["Category groups", "forum-category-groups", "Forum category groups"],
  ["Forum categories", "forum-categories", "Forum categories"],
  ["Forum threads", "forum-threads", "Forum threads"],
  ["Posts and posts layout", "forum-posts", "Forum posts and post layout"],
  [
    "Interaction of Pages and Forum",
    "page-forum-integration",
    "Page and forum integration",
  ],
];

for (const [heading, id, title] of siteStructureSections) {
  addFeature({
    id,
    title,
    category: "site-structure",
    summary: `Implement the documented Wikidot site-structure capability “${title}”, including its identity, relationships, routes, and rendering implications.`,
    sources: [sectionReference("doc:site-structure", heading)],
    seams: platformSeams,
  });
}

const layoutSections = [
  ["Page layout", "layout-page", "Default page layout"],
  ["Custom layout", "layout-custom", "Custom page layouts"],
  ["Forum structure", "layout-forum", "Forum layout structure"],
];

for (const [heading, id, title] of layoutSections) {
  addFeature({
    id,
    title,
    category: "layout",
    summary: `Render ${title.toLowerCase()} with the documented placeholders, conditional sections, element order, identifiers, and nesting.`,
    sources: [sectionReference("doc:layout-reference", heading)],
    seams: [...syntaxSeams, ...platformSeams],
  });
}

const marketingSectionMappings = new Map([
  [
    "PROFESSIONAL WIKI TECHNOLOGY",
    ["hosted-wiki-platform", "Hosted wiki platform"],
  ],
  ["SAFETY", ["service-resilience", "Service resilience and data safety"]],
  ["HOSTING", ["managed-hosting", "Managed site hosting"]],
  ["STORAGE", ["site-storage", "Site file storage"]],
  ["UNLIMITED NUMBER OF PAGES", ["unlimited-pages", "Unlimited site pages"]],
  ["CONTROL OVER ADS", ["advertising", "Site advertising"]],
  ["POWERFUL WIKI SYNTAX AND ENGINE", ["syntax-engine", "Wiki syntax engine"]],
  ["YOUR OWN DOMAIN", ["custom-domains", "Custom site domains"]],
  ["FORUM FOR EACH SITE", ["forums-overview", "Site forums"]],
  ["FORUM SIGNATURE", ["forum-signatures", "Forum signatures"]],
  ["AVATAR", ["avatars", "User avatars"]],
  ["GRAVATAR INTEGRATION", ["gravatar", "Gravatar integration"]],
  ["KARMA", ["karma", "User karma"]],
  ["PRIVATE MESSAGES", ["private-messages", "Private messages and contacts"]],
  [
    "EASY NAVIGATION AND USER INTERFACE",
    ["site-navigation", "Site navigation"],
  ],
  ["CATEGORIES", ["page-categories", "Page categories and namespaces"]],
  ["TAGS", ["page-tags", "Page tags"]],
  ["ROLES AND PERMISSIONS", ["roles-and-permissions", "Roles and permissions"]],
  ["MEMBERSHIP ON YOUR SITE", ["site-membership", "Site membership"]],
  ["THEMES", ["site-themes", "Site themes"]],
  ["LICENSE OF YOUR CONTENT", ["content-licensing", "Content licensing"]],
  ["SECURE SSL LOGIN", ["secure-login", "Secure login"]],
  ["SSL (HTTPS) ACCESS", ["site-https", "HTTPS site access"]],
  ["BACKUPS", ["site-backups", "Site backups"]],
  ["ADVANCED WEB STATISTICS", ["web-statistics", "Web statistics"]],
  ["FAVICONS", ["favicons", "Site favicons"]],
  ["EDITING OF <META> TAGS", ["meta-tags", "Site and page metadata tags"]],
  ["CLONING SITE", ["site-cloning", "Site cloning"]],
  [
    "CONTROLLING OUTGOING PINGBACKS",
    ["outgoing-pingbacks", "Outgoing pingbacks"],
  ],
]);

for (const [heading, [id, title]] of marketingSectionMappings) {
  const source = sectionReference("features", heading, "supporting");
  if (featureIds.has(id)) {
    const existing = features.find((feature) => feature.id === id);
    existing.sources.push(source);
    continue;
  }
  addFeature({
    id,
    title,
    category: "platform",
    summary: `Implement the documented Wikidot capability “${title}” and its user-visible configuration, state, permissions, and output.`,
    sources: [source],
    seams: platformSeams,
    documentationStatus: "high-level-documentation",
    implementationNotes: [
      "The corpus describes this capability at product level. Use live Wikidot evidence to resolve any implementation detail the snapshot does not define.",
    ],
  });
}

const faqFeatures = [
  {
    id: "page-editing-history",
    title: "Page editing modes and revision history",
    page: "faq:editing-pages",
    summary:
      "Provide Wikidot page editing modes, publishing behavior, source syntax workflow, and recoverable revision history.",
  },
  {
    id: "private-sites",
    title: "Private sites",
    page: "faq:private-sites",
    summary:
      "Enforce private-site visibility, membership access, unauthorized landing behavior, navigation exposure rules, and authenticated feed access.",
  },
  {
    id: "site-lifecycle-limits",
    title: "Site limits, backup, anti-abuse, deletion, and restoration",
    page: "faq:site-features",
    summary:
      "Implement the documented site ownership limits, storage/page limits, backup behavior, vandalism controls, founder-only deletion, and deletion undo.",
  },
  {
    id: "subscriptions-plans",
    title: "Subscriptions and account/site plans",
    page: "faq:upgrades",
    summary:
      "Represent Wikidot account and site upgrades, slots, storage limits, expiration, billing periods, administrator access, refunds, and payment rules.",
  },
  {
    id: "account-lifecycle",
    title: "User account lifecycle and authentication recovery",
    page: "faq:user-accounts",
    summary:
      "Support account eligibility, deletion, and documented recovery from authentication state problems.",
  },
  {
    id: "watching-notifications",
    title: "Watching and email notifications",
    page: "faq:watching",
    summary:
      "Allow users to watch and unwatch sites, categories, pages, and forum topics, with the documented inheritance and email notification behavior.",
  },
  {
    id: "browser-support",
    title: "Supported browsers",
    page: "faq:technical",
    summary:
      "Apply the documented browser-support policy to browser-visible Wikidot behavior.",
  },
];

for (const descriptor of faqFeatures) {
  addFeature({
    id: descriptor.id,
    title: descriptor.title,
    category: "platform",
    summary: descriptor.summary,
    sources: [lineReference(descriptor.page)],
    seams: platformSeams,
  });
}

addFeature({
  id: "community-site-directory",
  title: "Community Site directory and application",
  category: "platform",
  summary:
    "Represent Community Sites, their application and ownership rules, advertising rules, deletion constraints, and directory records stored as structured page data.",
  sources: [lineReference("community-sites"), lineReference("faq:community-sites")],
  seams: [...platformSeams, ...dataFormSeams],
  implementationNotes: [
    "The corpus contains 1,560 user-submitted `community-sites:*` application records. They are excluded from specifications and feature provenance; use the public application flow or a run-owned sandbox to establish any undocumented record shape.",
  ],
});

addFeature({
  id: "subscription-plan-matrix",
  title: "Subscription plan comparison",
  category: "platform",
  summary:
    "Display the documented plan capabilities, prices, limits, and comparison matrix.",
  sources: [lineReference("plans")],
  seams: platformSeams,
});

const moduleInvocationPattern = /\[\[module\s+([A-Za-z0-9_]+)/gi;
const moduleOccurrences = new Map();

for (const sourcePage of pages.values()) {
  if (classifySource(sourcePage).classification === "structured-data-record") {
    continue;
  }
  for (let index = 0; index < sourcePage.lines.length; index += 1) {
    const line = sourcePage.lines[index];
    moduleInvocationPattern.lastIndex = 0;
    let match;
    while ((match = moduleInvocationPattern.exec(line)) !== null) {
      const normalizedName = match[1].toLowerCase();
      const occurrences = moduleOccurrences.get(normalizedName) ?? {
        displayName: match[1],
        occurrences: [],
      };
      occurrences.occurrences.push({
        fullname: sourcePage.fullname,
        line: index + 1,
      });
      moduleOccurrences.set(normalizedName, occurrences);
    }
  }
}

for (const [normalizedName, occurrenceGroup] of [...moduleOccurrences].sort(
  ([left], [right]) => left.localeCompare(right),
)) {
  if (documentedModuleNames.has(normalizedName)) {
    continue;
  }
  const id = `module-${slugify(occurrenceGroup.displayName)}`;
  if (featureIds.has(id)) {
    continue;
  }
  const occurrenceSources = occurrenceGroup.occurrences
    .slice(0, 20)
    .map(({ fullname, line }) =>
      lineReference(fullname, line, line, "invocation-only"),
    );
  addFeature({
    id,
    title: `${titleCase(occurrenceGroup.displayName)} Module`,
    category: "module",
    summary: `Recognize and implement the \`${occurrenceGroup.displayName}\` module at the documented invocation sites. The corpus does not provide a dedicated module reference page.`,
    sources: occurrenceSources,
    seams: moduleSeams,
    documentationStatus: "invocation-only",
    implementationNotes: [
      "The documentation corpus proves the module name and invocation context, but not a complete behavior contract.",
      "Before implementing behavior beyond the recorded invocation, capture live Wikidot output at the public rendering or browser seam and add that evidence to this specification.",
    ],
  });
  documentedModuleNames.set(normalizedName, id);
}

const featureById = new Map(features.map((feature) => [feature.id, feature]));

function attachSource(featureId, source) {
  const feature = featureById.get(featureId);
  invariant(feature, `Cannot attach source to unknown feature ${featureId}`);
  const duplicate = feature.sources.some(
    (candidate) =>
      candidate.fullname === source.fullname &&
      candidate.start_line === source.start_line &&
      candidate.end_line === source.end_line,
  );
  if (!duplicate) {
    feature.sources.push(source);
  }
}

attachSource(
  "data-forms-overview",
  lineReference("doc:data-forms", 1, undefined, "redirect"),
);
attachSource(
  "data-forms-overview",
  lineReference("doc-data-forms:reference", 1, undefined, "supporting"),
);
attachSource(
  "page-templates",
  lineReference("doc-wiki-syntax:_template", 1, undefined, "template-example"),
);
attachSource(
  "syntax-engine",
  lineReference("doc-wiki-syntax:start", 1, undefined, "supporting"),
);
attachSource(
  "site-navigation",
  lineReference("nav:side", 1, undefined, "site-navigation-example"),
);
attachSource(
  "site-navigation",
  lineReference("nav:top", 1, undefined, "site-navigation-example"),
);

addFeature({
  id: "collaborative-editing",
  title: "Collaborative page and file editing",
  category: "platform",
  summary:
    "Allow authorized users to create and edit shared pages, publish changes, collaborate on documents, and share files through a site.",
  sources: [
    lineReference("inc:what-is-wikidot", 5, 6, "supporting"),
    lineReference("inc:awesome-features", 22, 30, "supporting"),
    lineReference("education", 20, 32, "supporting"),
  ],
  seams: platformSeams,
  documentationStatus: "high-level-documentation",
  implementationNotes: [
    "The corpus states the collaborative capability but does not define concurrent-edit conflict semantics. Capture live behavior before choosing a conflict model.",
  ],
});
featureById.set("collaborative-editing", features.at(-1));

addFeature({
  id: "educational-site-status",
  title: "Educational site status",
  category: "platform",
  summary:
    "Support the documented educational-site eligibility, application authority, storage, file-size, membership, revision, HTTPS, analytics, cost, and upgrade interaction rules.",
  sources: [lineReference("education", 43, 65)],
  seams: platformSeams,
  documentationStatus: "documented-plan-capability",
});
featureById.set("educational-site-status", features.at(-1));

const sourceFeatureMap = new Map(
  [...pages.keys()].map((fullname) => [fullname, new Set()]),
);

for (const feature of features) {
  feature.sources.sort(
    (left, right) =>
      left.fullname.localeCompare(right.fullname) ||
      left.start_line - right.start_line,
  );
  for (const source of feature.sources) {
    sourceFeatureMap.get(source.fullname).add(feature.id);
  }
}

for (const [normalizedName, occurrenceGroup] of moduleOccurrences) {
  const featureId = documentedModuleNames.get(normalizedName);
  if (!featureId) {
    continue;
  }
  for (const occurrence of occurrenceGroup.occurrences) {
    sourceFeatureMap.get(occurrence.fullname).add(featureId);
  }
}

for (const sourcePage of pages.values()) {
  const redirectMatch = sourcePage.source.match(
    /\[\[module\s+Redirect\s+destination=["']?([^"'\]\s]+)/i,
  );
  if (!redirectMatch) {
    continue;
  }
  const destination = redirectMatch[1].replace(/^\/+/, "").toLowerCase();
  const directFeature = features.find((feature) =>
    feature.sources.some(
      (source) => source.fullname.toLowerCase() === destination,
    ),
  );
  if (directFeature) {
    sourceFeatureMap.get(sourcePage.fullname).add(directFeature.id);
  }
}

for (const fullname of ["doc:quick-reference", "doc:quick-reference-mini"]) {
  for (const feature of features.filter(
    (candidate) => candidate.category === "wiki-syntax",
  )) {
    sourceFeatureMap.get(fullname).add(feature.id);
  }
}

for (const fullname of [
  "nav:doc",
  "nav:doc-data-forms",
  "nav:doc-modules",
  "nav:doc-wiki-syntax",
  "nav:topdoc",
]) {
  sourceFeatureMap.get(fullname).add("site-navigation");
}

function mapSupportingPage(fullname, featureIdsToMap) {
  for (const featureId of featureIdsToMap) {
    invariant(
      featureById.has(featureId),
      `Unknown supporting feature ${featureId}`,
    );
    sourceFeatureMap.get(fullname).add(featureId);
  }
}

mapSupportingPage("admin:themes", ["site-themes"]);
mapSupportingPage("advertise", ["advertising"]);
mapSupportingPage("files", ["module-files", "syntax-attachment"]);
mapSupportingPage("doc-data-forms:reference", ["data-forms-overview"]);
mapSupportingPage("doc-data-forms:thanks", ["data-forms-overview"]);
mapSupportingPage("doc:start", ["hosted-wiki-platform"]);
mapSupportingPage("doc:video", [
  "hosted-wiki-platform",
  "page-categories",
  "module-listpages",
  "syntax-tables",
  "module-css",
]);
mapSupportingPage("inc:how-it-works", [
  "account-lifecycle",
  "hosted-wiki-platform",
  "site-membership",
]);
mapSupportingPage("inc:awesome-features", [
  "collaborative-editing",
  "hosted-wiki-platform",
  "managed-hosting",
  "service-resilience",
  "subscriptions-plans",
]);
mapSupportingPage("more:explore-features", [
  ...new Set([...marketingSectionMappings.values()].map(([id]) => id)),
]);
mapSupportingPage("education", [
  "collaborative-editing",
  "educational-site-status",
  "forums-overview",
  "module-feed",
  "private-sites",
  "site-themes",
  "syntax-bibliography",
  "syntax-footnotes",
  "syntax-math",
]);

function classifySource(sourcePage) {
  const { fullname, source } = sourcePage;
  if (fullname.startsWith("community-sites:")) {
    return {
      classification: "structured-data-record",
      reason:
        "A user-submitted Community Site data-form record; excluded from compatibility evidence and individual repository documentation.",
    };
  }
  if (/\[\[module\s+Redirect\b/i.test(source)) {
    return {
      classification: "redirect-or-alias",
      reason:
        "A compatibility alias or redirect to a canonical documentation or runtime page.",
    };
  }
  if (
    fullname.startsWith("doc-wiki-syntax:") ||
    fullname.startsWith("doc-modules:") ||
    fullname.startsWith("doc-data-forms:") ||
    fullname.startsWith("doc-include:") ||
    fullname.startsWith("doc:")
  ) {
    return {
      classification: "documentation",
      reason:
        "A canonical documentation page, shared documentation fragment, reference page, or documentation index.",
    };
  }
  if (fullname.startsWith("faq:")) {
    return {
      classification: "documentation",
      reason: "A feature FAQ that records user-visible behavior or limits.",
    };
  }
  if (fullname.startsWith("legal:") || fullname === "ads-tos") {
    return {
      classification: "policy-not-feature",
      reason:
        "A legal or commercial policy page. It was inspected but does not define a discrete software feature contract.",
    };
  }
  if (fullname.startsWith("nav:")) {
    return {
      classification: "navigation-composition",
      reason:
        "A site navigation source page used as evidence for page composition and navigation features.",
    };
  }
  if (
    fullname.startsWith("forum:") ||
    fullname.startsWith("system:") ||
    fullname.startsWith("search:") ||
    [
      "_4040",
      "_maintenance",
      "account",
      "action:deleteaccount",
      "admin:manage",
      "admin:themes",
      "files",
      "invitation",
      "new-site",
      "search",
      "un",
      "user:info",
    ].includes(fullname)
  ) {
    return {
      classification: "runtime-system-page",
      reason:
        "A system route or generated page that invokes or composes a documented runtime feature.",
    };
  }
  if (
    fullname.startsWith("inc:") ||
    fullname.startsWith("more:") ||
    [
      "about",
      "ads",
      "advertise",
      "community-sites",
      "education",
      "features",
      "plans",
      "start",
      "start:start",
    ].includes(fullname)
  ) {
    return {
      classification: "product-or-presentation-page",
      reason:
        "A product, presentation, or composition page used as supporting evidence where it states a feature.",
    };
  }
  if (
    [
      "changelog",
      "community-sites",
      "doc",
      "files",
      "legal:privacy-policy",
      "legal:terms-of-service",
      "more:testimonials",
    ].includes(fullname)
  ) {
    return {
      classification: "index-or-non-feature-content",
      reason:
        "An index, empty placeholder, changelog, or testimonial page without a distinct feature contract.",
    };
  }
  return {
    classification: "index-or-non-feature-content",
    reason:
      "A corpus page that was inspected but contains presentation, account shell, or other content without an additional discrete feature contract.",
  };
}

function sourceRangeText(source) {
  const sourcePage = page(source.fullname);
  if (sourcePage.lineCount === 0) {
    return "(The source file is empty.)";
  }
  return sourcePage.lines
    .slice(source.start_line - 1, source.end_line)
    .map((line, offset) => {
      const lineNumber = source.start_line + offset;
      return `L${String(lineNumber).padStart(4, "0")} ${line}`;
    })
    .join("\n");
}

function renderImplementationContract(feature) {
  const categoryContracts = {
    "wiki-syntax": [
      "The parser MUST recognize every documented spelling and structural form in the evidence below.",
      "The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.",
      "Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.",
    ],
    module: [
      "The module dispatcher MUST recognize every documented module name and compatibility alias.",
      "The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.",
      "The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.",
    ],
    "data-forms": [
      "Category templates MUST recognize the documented field and layout syntax.",
      "Create and edit flows MUST validate, normalize, store, and redisplay field values as documented.",
      "Page rendering, template variables, CSS hooks, ListPages selection, and ordering MUST expose stored values as documented.",
    ],
    api: [
      "The public API MUST accept the documented method name and parameter forms.",
      "Authentication, authorization, limits, filtering, ordering, return shapes, and errors MUST match the documented contract.",
      "Deleted methods MUST remain unavailable unless live compatibility evidence proves a later replacement.",
    ],
    platform: [
      "The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.",
      "Account, site, category, page, and actor context MUST be enforced at the public service boundary.",
      "Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.",
    ],
    "site-structure": [
      "The persistence model MUST represent the documented entity and relationships.",
      "Public links, routes, selection behavior, permissions, and rendered structure MUST preserve those relationships.",
      "Imported Wikidot identifiers and URLs MUST remain compatibility-stable.",
    ],
    layout: [
      "The Wikidot layout renderer MUST emit the documented regions, identifiers, order, and nesting.",
      "Conditional regions and placeholders MUST use the documented context and visibility rules.",
      "Browser tests MUST verify final DOM and any user-visible intermediate state.",
    ],
  };
  return categoryContracts[feature.category] ?? platformSeams;
}

function validateDetailedContracts() {
  invariant(
    detailedContracts.schema === "wikijump.wikidot_detailed_feature_contracts.v1",
    "Unexpected detailed feature contract schema",
  );
  invariant(
    detailedContracts.evidence_manifest ===
      "docs/wikidot-specifications/detailed-spec-evidence-20260816.json",
    "Detailed feature contracts point at an unexpected evidence manifest",
  );
  invariant(
    detailedSpecEvidence.schema === "wikijump.wikidot_detailed_spec_evidence.v1",
    "Unexpected detailed specification evidence schema",
  );
  invariant(
    /^[0-9a-f]{40}$/.test(
      detailedContracts.source_gap_snapshot?.wikijump_commit ?? "",
    ),
    "Detailed feature contracts have no exact source-gap Wikijump commit",
  );
  invariant(
    detailedContracts.source_gap_snapshot?.canonical_surface_count === 870,
    "Detailed feature contracts have an unexpected canonical surface count",
  );
  invariant(
    detailedContracts.source_gap_snapshot?.feature_count === 57,
    "Detailed feature contracts must describe exactly 57 source-gap features",
  );
  invariant(
    detailedContracts.features &&
      !Array.isArray(detailedContracts.features) &&
      typeof detailedContracts.features === "object",
    "Detailed feature contracts features must be an object",
  );
  const contractIds = Object.keys(detailedContracts.features).sort();
  invariant(
    contractIds.length === detailedContracts.source_gap_snapshot.feature_count,
    "Detailed feature contract feature count does not match the source-gap snapshot",
  );
  const catalogIds = new Set(features.map(({ id }) => id));
  const axisIds = Object.keys(detailedContractAxes);
  let axisCount = 0;
  for (const featureId of contractIds) {
    invariant(
      catalogIds.has(featureId),
      `Detailed feature contract refers to unknown catalog feature ${featureId}`,
    );
    const contract = detailedContracts.features[featureId];
    invariant(
      contract && typeof contract === "object" && !Array.isArray(contract),
      `Detailed feature contract ${featureId} must be an object`,
    );
    invariant(
      JSON.stringify(Object.keys(contract).sort()) ===
        JSON.stringify(["axes", "evidence"]),
      `Detailed feature contract ${featureId} has unknown fields`,
    );
    invariant(
      Array.isArray(contract.evidence) &&
        contract.evidence.length > 0 &&
        new Set(contract.evidence).size === contract.evidence.length,
      `Detailed feature contract ${featureId} has invalid evidence aliases`,
    );
    for (const alias of contract.evidence) {
      const evidence = detailedEvidenceAliases[alias];
      invariant(
        evidence && typeof evidence === "object" && !Array.isArray(evidence),
        `Detailed feature contract ${featureId} uses unknown evidence alias ${alias}`,
      );
      invariant(
        typeof evidence.path === "string" && evidence.path.length > 0,
        `Detailed evidence ${alias} has no path`,
      );
      invariant(
        /^[0-9a-f]{64}$/.test(evidence.sha256 ?? ""),
        `Detailed evidence ${alias} has no SHA-256`,
      );
      const evidencePath = evidence.path.startsWith("/")
        ? evidence.path
        : join(repositoryRoot, evidence.path);
      const actual = readEvidenceFile(evidencePath);
      invariant(
        sha256(actual) === evidence.sha256,
        `Detailed evidence ${alias} hash drifted: ${evidence.path}`,
      );
    }
    invariant(
      contract.axes &&
        !Array.isArray(contract.axes) &&
        JSON.stringify(Object.keys(contract.axes)) === JSON.stringify(axisIds),
      `Detailed feature contract ${featureId} must define P1-P8 in order`,
    );
    for (const axisId of axisIds) {
      const requirements = contract.axes[axisId];
      invariant(
        Array.isArray(requirements) &&
          requirements.length > 0 &&
          new Set(requirements).size === requirements.length &&
          requirements.every(
            (value) =>
              typeof value === "string" &&
              value.trim() === value &&
              value.length >= 40,
          ),
        `Detailed feature contract ${featureId} ${axisId} has invalid requirements`,
      );
      axisCount += 1;
    }
  }
  invariant(
    axisCount === 57 * Object.keys(detailedContractAxes).length,
    "Detailed feature contract axis denominator is not exactly 456",
  );
  invariant(
    detailedContractsReferent.endsWith("\n"),
    "Detailed feature contract referent table must end with a newline",
  );
}

function renderDetailedContract(feature) {
  const contract = detailedContracts.features[feature.id];
  if (!contract) return "";
  const evidence = contract.evidence
    .map((alias) => {
      const item = detailedEvidenceAliases[alias];
      const claim = typeof item.claim === "string" ? `: ${item.claim}` : "";
      return `- \`${alias}\` -> \`${item.path}\` (SHA-256 \`${item.sha256}\`)${claim}`;
    })
    .join("\n");
  const axes = Object.entries(detailedContractAxes)
    .map(([axisId, title]) => `### ${axisId} - ${title}

${contract.axes[axisId].map((requirement) => `- ${requirement}`).join("\n")}`)
    .join("\n\n");
  return `
## Detailed conformance contract

- Status: \`detailed-p1-p8\`
- Source-gap snapshot: Wikijump \`${detailedContracts.source_gap_snapshot.wikijump_commit}\`
- Evidence manifest: \`${detailedContracts.evidence_manifest}\`

This section is normative. It maps the complete evidence below to every P1-P8
implementation axis. A statement that deliberately keeps an unobserved path
fail-closed is a boundary of the specification, not permission to invent the
missing Wikidot behavior.

Evidence basis:

${evidence}

${axes}
`;
}

function specificationPath(feature) {
  return join("specifications", feature.category, `${feature.id}.md`);
}

function renderSpecification(feature) {
  const sourceList = feature.sources
    .map(
      (source) =>
        `- \`${corpusPath(source.fullname)}:${source.start_line}\` through line ${source.end_line} (${source.role})`,
    )
    .join("\n");
  const seamList = feature.suggested_tdd_seams
    .map((seam) => `- ${seam}`)
    .join("\n");
  const requirements = renderImplementationContract(feature)
    .map((requirement) => `- ${requirement}`)
    .join("\n");
  const notes =
    feature.implementation_notes.length === 0
      ? "- No feature-specific implementation note beyond the corpus contract."
      : feature.implementation_notes.map((note) => `- ${note}`).join("\n");
  const evidence = feature.sources
    .map(
      (source) => `### ${source.fullname} (${source.role})

Source: \`${corpusPath(source.fullname)}:${source.start_line}\` through line ${source.end_line}  
SHA-256 of complete source file: \`${page(source.fullname).sha256}\`

\`\`\`wikidot
${sourceRangeText(source)}
\`\`\``,
    )
    .join("\n\n");
  const featureLiveObservations = liveObservations.observations.filter(
    (observation) => observation.feature_ids.includes(feature.id),
  );
  const featureSupplements = specSupplements.supplements.filter(
    (supplement) => supplement.feature_id === feature.id,
  );
  const detailedStatus = detailedContracts.features[feature.id]
    ? "- Detailed conformance status: `detailed-p1-p8`\n"
    : "";
  const liveEvidence =
    featureLiveObservations.length === 0 && featureSupplements.length === 0
      ? ""
      : `
## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

${featureLiveObservations
  .map(
    (observation) => `### ${observation.title}

- Observation ID: \`${observation.id}\`
- Classification: \`${observation.classification}\`
- Observed at: \`${observation.observed_at}\`
- Analysis: ${observation.analysis}

Normative behavior:

${observation.normative_behavior.map((claim) => `- ${claim}`).join("\n")}

${
  observation.raw_captures?.length > 0
    ? `Raw HTTP captures:

${observation.raw_captures
  .map(
    (capture) =>
      `- ${capture.phase} stored locale \`${capture.stored_locale}\`: \`${capture.path}\` (SHA-256 \`${capture.sha256}\`)`,
  )
  .join("\n")}

`
    : ""
}Evidence:

${observation.evidence
  .map(
    (item) => {
      const cases =
        item.case_ids.length === 0
          ? "none"
          : item.case_ids
              .map((caseId) => `\`${caseId}\``)
              .join(", ");
      return `- \`${item.path}\` (SHA-256 \`${item.sha256}\`), cases: ${cases}`;
    },
  )
  .join("\n")}
`,
  )
  .join("\n")}
${featureSupplements
  .map((supplement) => `
### ${supplement.title}

${supplement.markdown}
`)
  .join("")}`;

  return `# ${feature.title}

- Feature ID: \`${feature.id}\`
- Category: \`${feature.category}\`
- Documentation status: \`${feature.documentation_status}\`
${detailedStatus}- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

${feature.summary}

## Implementation contract

${requirements}

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.
${renderDetailedContract(feature)}${liveEvidence}

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

${seamList}

## Feature-specific implementation notes

${notes}

## Source inventory

${sourceList}

## Documentation-derived behavioral evidence

${evidence}
`;
}

validateDetailedContracts();

features.sort(
  (left, right) =>
    left.category.localeCompare(right.category) ||
    left.id.localeCompare(right.id),
);

const specificationFiles = new Map();
for (const feature of features) {
  const relativePath = specificationPath(feature);
  invariant(
    !specificationFiles.has(relativePath),
    `Duplicate specification path: ${relativePath}`,
  );
  specificationFiles.set(relativePath, renderSpecification(feature));
}

const classifiedPages = [...pages.values()]
  .sort((left, right) => left.fullname.localeCompare(right.fullname))
  .map((sourcePage) => ({ sourcePage, classification: classifySource(sourcePage) }));

const coverageEntries = classifiedPages
  .filter(
    ({ classification }) =>
      classification.classification !== "structured-data-record",
  )
  .map(({ sourcePage, classification }) => ({
    fullname: sourcePage.fullname,
    title: sourcePage.metadata.title ?? "",
    source_path: corpusPath(sourcePage.fullname),
    source_sha256: sourcePage.sha256,
    source_bytes: Buffer.byteLength(sourcePage.source),
    source_lines: sourcePage.lineCount,
    classification: classification.classification,
    classification_reason: classification.reason,
    feature_ids: [...sourceFeatureMap.get(sourcePage.fullname)].sort(),
  }));

const excludedDataGroups = [...new Set(
  classifiedPages
    .filter(
      ({ classification }) =>
        classification.classification === "structured-data-record",
    )
    .map(({ classification }) => classification.classification),
)]
  .sort()
  .map((classificationName) => {
    const group = classifiedPages.filter(
      ({ classification }) => classification.classification === classificationName,
    );
    return {
      classification: classificationName,
      classification_reason: group[0].classification.reason,
      path_prefix: "community-sites:",
      page_count: group.length,
      source_bytes: group.reduce(
        (total, { sourcePage }) => total + Buffer.byteLength(sourcePage.source),
        0,
      ),
      source_inventory_sha256: sha256(
        group
          .map(({ sourcePage }) => `${sourcePage.fullname}\t${sourcePage.sha256}`)
          .join("\n"),
      ),
    };
  });

const classificationCounts = Object.fromEntries(
  [...new Set(
    classifiedPages.map(({ classification }) => classification.classification),
  )]
    .sort()
    .map((classification) => [
      classification,
      classifiedPages.filter(
        ({ classification: candidate }) => candidate.classification === classification,
      ).length,
    ]),
);
const sourcePagesWithFeatures = classifiedPages.filter(
  ({ sourcePage }) => sourceFeatureMap.get(sourcePage.fullname).size > 0,
).length;
const sourcePagesWithoutFeatures = pages.size - sourcePagesWithFeatures;

const catalog = {
  schema_version: schemaVersion,
  generated_date: generatedDate,
  language: "English",
  corpus: {
    root: displayedCorpusRoot,
    expanded_root: corpusRoot,
    page_count: pages.size,
    source_file_count: pages.size,
    listed_source_file_count: coverageEntries.length,
    excluded_data_record_count: excludedDataGroups.reduce(
      (total, group) => total + group.page_count,
      0,
    ),
    source_bytes: classifiedPages.reduce(
      (total, { sourcePage }) => total + Buffer.byteLength(sourcePage.source),
      0,
    ),
    classification_counts: classificationCounts,
    unclassified_count: 0,
    source_pages_with_features: sourcePagesWithFeatures,
    source_pages_without_features: sourcePagesWithoutFeatures,
    coverage_file: "source-coverage.json",
  },
  conventions: {
    feature_granularity:
      "One catalog item and one Markdown file per independently implementable syntax feature, module, data-form capability, API method, site-structure behavior, layout behavior, or platform/runtime capability.",
    authority:
      "The files are exhaustive extractions of the frozen documentation corpus. They are not claims that the snapshot is complete or correct. Reproducible live Wikidot behavior overrides a conflicting documentation claim.",
    partial_documentation:
      "invocation-only, high-level-documentation, and partially-documented items require live-oracle evidence before unspecified behavior is invented.",
  },
  live_observations: {
    observation_count: liveObservations.observations.length,
    source_file: "live-observations.json",
  },
  feature_count: features.length,
  categories: Object.fromEntries(
    [...new Set(features.map((feature) => feature.category))]
      .sort()
      .map((category) => [
        category,
        features.filter((feature) => feature.category === category).length,
      ]),
  ),
  features: features.map((feature) => ({
    id: feature.id,
    title: feature.title,
    category: feature.category,
    documentation_status: feature.documentation_status,
    specification: specificationPath(feature),
    summary: feature.summary,
    source_count: feature.sources.length,
    sources: feature.sources.map((source) => ({
      path: corpusPath(source.fullname),
      start_line: source.start_line,
      end_line: source.end_line,
      role: source.role,
      source_sha256: page(source.fullname).sha256,
    })),
    suggested_tdd_seams: feature.suggested_tdd_seams,
    related_features: feature.related_features,
    live_observation_ids: liveObservations.observations
      .filter((observation) => observation.feature_ids.includes(feature.id))
      .map((observation) => observation.id),
  })),
};
const serializedCatalog = `${JSON.stringify(catalog, null, 2)}\n`;
const rawImplementationLedger = readFileSync(
  implementationLedgerSourcePath,
  "utf8",
);
const implementationLedger = JSON.parse(rawImplementationLedger);
validateWikidotImplementationLedger({
  ledger: implementationLedger,
  rawCatalog: serializedCatalog,
  catalog,
  liveObservationIds: [...liveObservationIds],
  repositoryRoot,
});

const coverage = {
  schema_version: schemaVersion,
  generated_date: generatedDate,
  language: "English",
  corpus_root: displayedCorpusRoot,
  page_count: pages.size,
  listed_page_count: coverageEntries.length,
  excluded_data_record_count: excludedDataGroups.reduce(
    (total, group) => total + group.page_count,
    0,
  ),
  unclassified_count: 0,
  classification_counts: classificationCounts,
  source_pages_with_features: sourcePagesWithFeatures,
  source_pages_without_features: sourcePagesWithoutFeatures,
  excluded_data_groups: excludedDataGroups,
  pages: coverageEntries,
};

const catalogRows = features
  .map(
    (feature) =>
      `| \`${feature.id}\` | ${escapeMarkdownTableCell(feature.title)} | \`${feature.documentation_status}\` | [specification](${specificationPath(feature)}) |`,
  )
  .join("\n");
const categorySummary = Object.entries(catalog.categories)
  .map(([category, count]) => `- \`${category}\`: ${count}`)
  .join("\n");
const detailedSourceGapFeatures = features.filter(
  (feature) => detailedContracts.features[feature.id],
);
const detailedSourceGapSections = [...new Set(
  detailedSourceGapFeatures.map((feature) => feature.category),
)]
  .sort()
  .map((category) => {
    const rows = detailedSourceGapFeatures
      .filter((feature) => feature.category === category)
      .map(
        (feature) =>
          `- [${feature.title}](${specificationPath(feature)}) (\`${feature.id}\`, \`${feature.documentation_status}\`)`,
      )
      .join("\n");
    return `## ${category}\n\n${rows}`;
  })
  .join("\n\n");
const detailedSourceGapIndex = `# Detailed P1-P8 compatibility contract library

This is the human-readable table of contents for the 57 features hardened in the 2026-08-16 source-gap snapshot whose generated specifications contain a normative \`detailed-p1-p8\` conformance contract. It is a stable contract library, not the live implementation queue: select current work from the canonical compatibility ledger, then use this index when the selected feature has a hardened contract.

- Machine-readable contract set: [detailed-feature-contracts.json](detailed-feature-contracts.json)
- Evidence manifest: [detailed-spec-evidence-20260816.json](detailed-spec-evidence-20260816.json)
- Complete 210-feature catalog: [CATALOG.md](CATALOG.md)

${detailedSourceGapSections}
`;
const catalogMarkdown = `# Wikidot feature catalog

This is the human-readable index of every feature extracted from the frozen local Wikidot documentation corpus. The authoritative machine-readable form is [catalog.json](catalog.json); source-page disposition is recorded in [source-coverage.json](source-coverage.json).

## Summary

- Features: ${catalog.feature_count}
- Corpus pages enumerated: ${pages.size}
- Corpus pages listed individually: ${coverageEntries.length}
- User data records excluded from repository documentation: ${coverage.excluded_data_record_count}
- Corpus pages connected to one or more feature IDs: ${sourcePagesWithFeatures}
- Corpus pages classified without a feature ID: ${sourcePagesWithoutFeatures}
- Unclassified corpus pages: 0
- Hardened P1-P8 snapshot contracts: ${Object.keys(detailedContracts.features).length}
- Hardened contract navigation: [P1-P8 contract library](DETAILED_SOURCE_GAP_SPECIFICATIONS.md)

Features by category:

${categorySummary}

## Status meanings

- \`documented\`: the snapshot contains a direct behavioral reference.
- \`documented-deprecated\`: the behavior is documented but explicitly deprecated.
- \`documented-negative\`: the documented behavior is that an interface is absent or removed.
- \`documented-plan-capability\`: the behavior is tied to a documented account/site plan.
- \`high-level-documentation\`: the feature is stated, but implementation details require live-oracle work.
- \`partially-documented\`: the canonical page is empty or incomplete.
- \`invocation-only\`: the corpus proves a module name and use site but has no dedicated contract.

## Features

| Feature ID | Title | Documentation status | Specification |
|---|---|---|---|
${catalogRows}
`;

const readme = `# Wikidot feature specifications

This directory is an exhaustive, documentation-derived implementation inventory for the frozen Wikidot corpus at \`${displayedCorpusRoot}\`.

- \`catalog.json\` is the authoritative machine-readable feature index.
- \`CATALOG.md\` is the human-readable index.
- \`DETAILED_SOURCE_GAP_SPECIFICATIONS.md\` is the human-readable library of the 57 features hardened in the 2026-08-16 source-gap snapshot, linking directly to each normative P1-P8 specification. It is not the live work queue.
- \`source-coverage.json\` proves that all ${pages.size.toLocaleString("en-US")} corpus pages were enumerated and classified, while listing only non-user pages individually.
- \`live-observations.json\` records reproducible live-Wikidot corrections that override conflicting or incomplete corpus claims.
- \`implementation-ledger.json\` tracks status, seams, tests, implementation files, evidence, blockers, and the campaign's P1-P8 feature-property matrix.
- \`detailed-feature-contracts.json\` is the machine-readable P1-P8 contract set for the 57 features hardened in that snapshot against current Wikidot evidence.
- \`detailed-spec-evidence-20260816.json\` seals the documentation, live Wikidot, and retained evidence used by those detailed contracts without storing credentials or private message content.
- \`specifications/\` contains exactly one English Markdown specification for every catalog item.
- \`IMPLEMENTATION_PROMPT.md\` gives a coding agent the vertical-slice TDD mechanics for current ledger-selected compatibility work.

## Interpretation rules

1. A corpus page is not automatically a feature. The 1,560 \`community-sites:*\` pages are user-submitted application records, not compatibility evidence. They are excluded from specifications and individual coverage entries; coverage keeps only an aggregate count, byte total, and inventory digest.
2. Redirects, indexes, navigation fragments, marketing repetitions, policies, and runtime composition pages are retained in \`source-coverage.json\`; relevant pages are attached to canonical feature specs as supporting evidence.
3. Every normative source extract retains its exact corpus page, original line numbers, and complete-file SHA-256.
4. Documentation status matters. \`invocation-only\`, \`high-level-documentation\`, and \`partially-documented\` specs identify real features but do not authorize invented behavior.
5. This snapshot is a specification-discovery input. When a reproducible live Wikidot observation conflicts with it, record both and implement live behavior.
6. A specification marked \`detailed-p1-p8\` has explicit normative coverage for all P1-P8 axes. A fail-closed statement in that section is an intentional compatibility boundary, not permission to infer the missing behavior from local output.

## Regeneration

\`\`\`bash
node scripts/generate-wikidot-specifications.mjs
node scripts/generate-wikidot-specifications.mjs --check
\`\`\`

Set \`WIKIDOT_DOCUMENTATION_CORPUS\` only when regenerating from a different checkout of the same corpus layout.
`;

const implementationPrompt = `# Prompt: implement current Wikidot compatibility work with TDD

This directory is the complete feature specification set. Select implementation work from the current canonical compatibility ledger and current blocker/issue authority, then use the matching catalog specification with test-driven development. Do not treat the catalog size or the hardened-contract snapshot as a mutable progress queue, and do not create one pull request per feature.

## Inputs

1. Read the repository's \`AGENTS.md\` completely.
2. Read the current canonical compatibility ledger to select the current row/dimension. Then resolve that row in \`docs/wikidot-specifications/catalog.json\`, which is the complete feature index rather than the live queue.
3. Read \`docs/wikidot-specifications/CATALOG.md\` and \`docs/wikidot-specifications/README.md\`. \`DETAILED_SOURCE_GAP_SPECIFICATIONS.md\` is the hardened P1-P8 contract library from its frozen snapshot; use it when the selected current feature is present there.
4. For the selected catalog item, read the exact Markdown file named by its \`specification\` field before designing or changing code.
5. Use \`docs/wikidot-specifications/source-coverage.json\` to inspect corroborating, redirect, runtime-composition, and non-feature source classifications when provenance is relevant. User-submitted data-record groups are aggregate-only and are never behavioral evidence.
6. Follow the repository architecture boundaries: FTML owns syntax parsing and rendering primitives; Wikijump/Deepwell owns site, page, query, import, file, permission, actor, module evaluation, and URL state; Framerail owns HTTP and browser runtime behavior.

## Authority and ambiguity

- Implement the documented contract, including legacy names, aliases, defaults, limits, output structure, URLs, permissions, side effects, and stated limitations.
- Do not modernize compatibility-sensitive syntax, DOM, identifiers, or routes.
- Live Wikidot is the behavioral oracle when the snapshot is ambiguous, incomplete, contradictory, or wrong.
- For an \`invocation-only\`, \`high-level-documentation\`, or \`partially-documented\` item, do not invent missing semantics. Design a minimal live-oracle experiment, preserve the evidence and exact fixture, update the specification, and then implement the observed behavior.
- Unsupported or unverified input must fail closed, remain literal, or use an evidenced fallback. It must not silently broaden queries or permissions.
- Once live evidence establishes a behavior, reproduce that boundary even when it is less defensive than a modern implementation would normally choose. Do not keep a stricter security or resource behavior merely as hardening. Record concerning oracle behavior in \`docs/wikidot-compatibility-security.md\`; any stricter intentional divergence requires an explicit product decision and is not exact parity.

## Mandatory TDD process

Before writing a test, state the seam map for the current vertical slice: the public interface being tested, the authority for the expected behavior, and why that seam is the appropriate observable boundary. Proceed when current authority is sufficient. Seek external/human authority only when the behavior or product/security decision is genuinely underdetermined; ordinary implementation must not stop for ceremonial confirmation. Suggested seams in each spec are recommendations, not pre-approval.

Then repeat this loop for each selected current behavior:

1. Select one small, user-observable vertical slice.
2. Write one behavior-focused test through the confirmed public seam.
3. Use an independent expected value from the specification or captured live Wikidot evidence.
4. Run the test and demonstrate that it fails for the intended missing behavior (red).
5. Write only enough production code to satisfy that test (green).
6. Run the focused test and the nearest affected suite.
7. Continue with the next learned behavior. Do not write all tests first and all implementation later.

Tests must describe what callers or users observe and must survive internal refactors. Do not test private methods, internal call counts, or database rows through a side channel when a public read interface exists. Do not mock code owned by the repository. Mock only true system boundaries when unavoidable; prefer the real parser, renderer, test database, HTTP route, and browser runtime.

Refactoring is a review-stage activity after a coherent set of red→green slices, not a speculative step inside the loop.

## Required coverage per catalog item

For every item, cover all documented:

- valid syntax and ordinary behavior;
- aliases, legacy spellings, defaults, omitted and empty values;
- limits, boundary values, malformed values, and documented fallbacks;
- argument and feature interactions;
- permissions, visibility, actor, page, category, and site context;
- output text, DOM structure, IDs, classes, links, routes, and side effects;
- escaping, sanitization, and literal/fail-closed boundaries;
- URL, reload, direct navigation, back/forward, and client-runtime behavior where applicable;
- examples and stated limitations.

Add regression tests for every discovered defect. Preserve the original failing input and minimize fuzz or mutation failures into stable fixtures without losing provenance.

## Work tracking

Maintain the existing machine-readable implementation ledger keyed by \`catalog.json\` feature ID; do not create a second progress ledger. Each entry records:

- status: \`pending\`, \`in_progress\`, \`implemented\`, or \`blocked\`;
- confirmed public seams;
- test files and test names;
- implementation files;
- documentation and live-oracle evidence used;
- unresolved ambiguities or blockers.

Treat compatibility coverage as a feature-by-property matrix. Every feature in \`campaign.requested_scope\` must classify all eight axes:

- P1 invocation grammar and scalar interpretation;
- P2 parser stage, nesting, and composition;
- P3 lifecycle, persistence, import, and round trips;
- P4 actors, permissions, visibility, and privacy;
- P5 selection, ordering, counting, and pagination;
- P6 HTTP, API, URL, Ajax, feed, and navigation contracts;
- P7 DOM, CSS, resources, interaction, and geometry;
- P8 temporal behavior, failure atomicity, limits, and resource bounds.

Each property must be \`evidence_backed\`, \`documentation_only\`, \`unobserved\`, \`blocked\`, or \`not_applicable\`, with durable evidence or an exact observation gap. \`evidence_backed\` requires both a canonical \`live:<observation-id>\` reference and a public \`test:<repository-path>#<exact-anchor>\` regression seam. Test paths and anchors must resolve inside the repository; local output, an internal-only unit seam, a fabricated reference, or a manual check alone is insufficient. There must be exactly one ledger entry per catalog item. An item is not \`implemented\` merely because adjacent code exists: every P1-P8 property must be either evidence-backed or explicitly inapplicable, and no property may retain an observation gap or rely only on a manual check.

Keep the work in one focused campaign and normal review sequence unless repository ownership boundaries require a deliberately coordinated FTML change. Do not split routine discoveries into one pull request per example or per catalog item.

## Validation and completion

Run focused tests during each slice, then run formatting, linting, clippy/build checks, relevant integration suites, verifier suites, and browser tests in proportion to the changed surfaces. For browser-visible behavior, use the repository-owned frozen browser oracle and local standing/candidate runtime; ordinary validation must not reacquire Wikidot. If a genuinely unknown Wikidot behavior must be learned, perform one explicit reviewed acquisition, reduce it to a checked-in fixture/assertion, and test that fixture offline thereafter.

The historical compatibility campaign is retired. Its final-zero receipt is provenance for the frozen baseline, not mutable execution authority and not a reason to depend on an old WJLab tree. Feature work is complete only when its current ledger dimensions and repository-owned executable regression seams are satisfied. Keep generated specification validation and the hermetic compatibility gate green, classify every discovered differential or fuzz result, leave no known reproducible gap without a fix or concrete blocker, and use the normal review/merge/standing process without force or admin merge.

A merge is not a deployment. After browser-visible changes, refresh the standing runtime and verify the served URL before reporting the behavior fixed.
`;

const expectedFiles = new Map([
  ["README.md", readme],
  ["CATALOG.md", catalogMarkdown],
  ["DETAILED_SOURCE_GAP_SPECIFICATIONS.md", detailedSourceGapIndex],
  ["catalog.json", serializedCatalog],
  [
    "detailed-feature-contracts.json",
    `${JSON.stringify(detailedContracts, null, 2)}\n`,
  ],
  [
    "detailed-spec-evidence-20260816.json",
    `${JSON.stringify(detailedSpecEvidence, null, 2)}\n`,
  ],
  ["referent-table-detailed-feature-contracts.md", detailedContractsReferent],
  [
    "referent-table-detailed-feature-contracts.sha256",
    `${sha256(detailedContractsReferent)}  referent-table-detailed-feature-contracts.md\n`,
  ],
  ["source-coverage.json", `${JSON.stringify(coverage, null, 2)}\n`],
  ["live-observations.json", `${JSON.stringify(liveObservations, null, 2)}\n`],
  [
    "implementation-ledger.json",
    rawImplementationLedger,
  ],
  ["IMPLEMENTATION_PROMPT.md", implementationPrompt],
  ...specificationFiles,
]);

function validateGeneratedFiles() {
  invariant(
    catalog.feature_count === specificationFiles.size,
    "Catalog and specification file counts differ",
  );
  invariant(
    new Set(catalog.features.map((feature) => feature.id)).size ===
      catalog.feature_count,
    "Catalog feature IDs are not unique",
  );
  invariant(
    coverage.pages.length + coverage.excluded_data_record_count === pages.size,
    "Source coverage totals do not include every corpus page",
  );
  invariant(
    coverage.unclassified_count === 0,
    "Unclassified source pages remain",
  );
  for (const feature of features) {
    for (const source of feature.sources) {
      invariant(
        classifySource(page(source.fullname)).classification !==
          "structured-data-record",
        `Feature ${feature.id} references a user data record`,
      );
    }
  }
  for (const feature of catalog.features) {
    invariant(
      specificationFiles.has(feature.specification),
      `Missing specification for ${feature.id}`,
    );
  }
}

validateGeneratedFiles();

if (checkOnly) {
  const actualFiles = [];
  function walk(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        walk(path);
      } else {
        actualFiles.push(relative(outputRoot, path));
      }
    }
  }
  walk(outputRoot);
  const expectedPaths = [...expectedFiles.keys()].sort();
  const actualPaths = actualFiles.sort();
  invariant(
    JSON.stringify(actualPaths) === JSON.stringify(expectedPaths),
    "Generated file set differs from expected output; run the generator",
  );
  for (const [path, expected] of expectedFiles) {
    const actual = readFileSync(join(outputRoot, path), "utf8");
    invariant(
      actual === expected,
      `Generated file is stale: docs/wikidot-specifications/${path}`,
    );
  }
  console.log(
    `Validated ${catalog.feature_count} specifications and ${coverage.page_count} corpus pages.`,
  );
  process.exit(0);
}

if (statSync(repositoryRoot).isDirectory()) {
  rmSync(outputRoot, { recursive: true, force: true });
  for (const [path, content] of expectedFiles) {
    const target = join(outputRoot, path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, content, "utf8");
  }
}

console.log(
  `Generated ${catalog.feature_count} specifications from ${coverage.page_count} corpus pages.`,
);
