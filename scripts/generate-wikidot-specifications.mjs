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

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..");
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
const corpusRoot = resolve(
  process.env.WIKIDOT_DOCUMENTATION_CORPUS ??
    "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages",
);
const displayedCorpusRoot =
  "~/src/Rokurolize/scp-wiki-translation/corpus/www/pages";

const checkOnly = process.argv.includes("--check");
const schemaVersion = 1;
const generatedDate = "2026-07-28";

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
  for (const evidence of observation.evidence) {
    const evidencePath = resolve(repositoryRoot, evidence.path);
    const rawEvidence = readFileSync(evidencePath, "utf8");
    invariant(
      sha256(rawEvidence) === evidence.sha256,
      `Live evidence hash drifted for ${observation.id}: ${evidence.path}`,
    );
    const evidenceRows = evidence.path.endsWith(".jsonl")
      ? rawEvidence
          .split(/\r?\n/)
          .filter(Boolean)
          .map((line) => JSON.parse(line))
      : [JSON.parse(rawEvidence)];
    const capturedCaseIds = new Set();
    for (const row of evidenceRows) {
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
    }
    for (const caseId of evidence.case_ids) {
      invariant(
        capturedCaseIds.has(caseId),
        `Live evidence case ${caseId} is missing from ${evidence.path}`,
      );
    }
  }
}

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
      sourcePage.lineCount === 0
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
  sources: [
    lineReference("community-sites"),
    lineReference("faq:community-sites"),
    lineReference("community-sites:1", 1, 1, "representative-data-record"),
    lineReference("community-sites:1", 3, 6, "representative-data-record"),
  ],
  seams: [...platformSeams, ...dataFormSeams],
  implementationNotes: [
    "The corpus contains 1,560 `community-sites:*` records. The representative ranges document non-free-text record fields; source-coverage.json inventories every record without copying user-submitted descriptions or contact details into the specification.",
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

for (const fullname of pages.keys()) {
  if (fullname.startsWith("community-sites:")) {
    sourceFeatureMap.get(fullname).add("community-site-directory");
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
        "A Community Site data-form record; evidence for the directory record shape, not a distinct platform feature.",
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
  const liveEvidence =
    featureLiveObservations.length === 0
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

Evidence:

${observation.evidence
  .map(
    (item) =>
      `- \`${item.path}\` (SHA-256 \`${item.sha256}\`), cases: ${item.case_ids
        .map((caseId) => `\`${caseId}\``)
        .join(", ")}`,
  )
  .join("\n")}
`,
  )
  .join("\n")}
`;

  return `# ${feature.title}

- Feature ID: \`${feature.id}\`
- Category: \`${feature.category}\`
- Documentation status: \`${feature.documentation_status}\`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

${feature.summary}

## Implementation contract

${requirements}

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.
${liveEvidence}

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

const coverageEntries = [...pages.values()]
  .sort((left, right) => left.fullname.localeCompare(right.fullname))
  .map((sourcePage) => {
    const classification = classifySource(sourcePage);
    return {
      fullname: sourcePage.fullname,
      title:
        classification.classification === "structured-data-record"
          ? "Community Site data record"
          : (sourcePage.metadata.title ?? ""),
      source_path: corpusPath(sourcePage.fullname),
      source_sha256: sourcePage.sha256,
      source_bytes: Buffer.byteLength(sourcePage.source),
      source_lines: sourcePage.lineCount,
      classification: classification.classification,
      classification_reason: classification.reason,
      feature_ids: [...sourceFeatureMap.get(sourcePage.fullname)].sort(),
    };
  });

const classificationCounts = Object.fromEntries(
  [...new Set(coverageEntries.map((entry) => entry.classification))]
    .sort()
    .map((classification) => [
      classification,
      coverageEntries.filter((entry) => entry.classification === classification)
        .length,
    ]),
);
const sourcePagesWithFeatures = coverageEntries.filter(
  (entry) => entry.feature_ids.length > 0,
).length;
const sourcePagesWithoutFeatures =
  coverageEntries.length - sourcePagesWithFeatures;

const catalog = {
  schema_version: schemaVersion,
  generated_date: generatedDate,
  language: "English",
  corpus: {
    root: displayedCorpusRoot,
    expanded_root: corpusRoot,
    page_count: pages.size,
    source_file_count: coverageEntries.length,
    source_bytes: coverageEntries.reduce(
      (total, entry) => total + entry.source_bytes,
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
const implementationLedger = JSON.parse(
  readFileSync(implementationLedgerSourcePath, "utf8"),
);
invariant(
  implementationLedger.schema === "wikijump.wikidot_implementation_ledger.v1",
  "Unexpected implementation ledger schema",
);
invariant(
  implementationLedger.catalog_sha256 === sha256(serializedCatalog),
  `Implementation ledger catalog hash is stale; expected ${sha256(serializedCatalog)}`,
);
invariant(
  JSON.stringify(Object.keys(implementationLedger.features).sort()) ===
    JSON.stringify(catalog.features.map((feature) => feature.id).sort()),
  "Implementation ledger must contain exactly one entry per catalog feature",
);

const coverage = {
  schema_version: schemaVersion,
  generated_date: generatedDate,
  language: "English",
  corpus_root: displayedCorpusRoot,
  page_count: coverageEntries.length,
  unclassified_count: 0,
  classification_counts: classificationCounts,
  source_pages_with_features: sourcePagesWithFeatures,
  source_pages_without_features: sourcePagesWithoutFeatures,
  pages: coverageEntries,
};

const catalogRows = features
  .map(
    (feature) =>
      `| \`${feature.id}\` | ${feature.title.replace(/\|/g, "\\|")} | \`${feature.documentation_status}\` | [specification](${specificationPath(feature)}) |`,
  )
  .join("\n");
const categorySummary = Object.entries(catalog.categories)
  .map(([category, count]) => `- \`${category}\`: ${count}`)
  .join("\n");
const catalogMarkdown = `# Wikidot feature catalog

This is the human-readable index of every feature extracted from the frozen local Wikidot documentation corpus. The authoritative machine-readable form is [catalog.json](catalog.json); source-page disposition is recorded in [source-coverage.json](source-coverage.json).

## Summary

- Features: ${catalog.feature_count}
- Corpus pages enumerated: ${pages.size}
- Corpus pages connected to one or more feature IDs: ${sourcePagesWithFeatures}
- Corpus pages classified without a feature ID: ${sourcePagesWithoutFeatures}
- Unclassified corpus pages: 0

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
- \`source-coverage.json\` proves that all ${pages.size.toLocaleString("en-US")} corpus pages were enumerated and classified.
- \`live-observations.json\` records reproducible live-Wikidot corrections that override conflicting or incomplete corpus claims.
- \`implementation-ledger.json\` tracks status, seams, tests, implementation files, evidence, and blockers for every catalog feature.
- \`specifications/\` contains exactly one English Markdown specification for every catalog item.
- \`IMPLEMENTATION_PROMPT.md\` instructs a coding agent to implement the complete catalog using vertical-slice TDD.

## Interpretation rules

1. A corpus page is not automatically a feature. The 1,560 \`community-sites:*\` pages, for example, are structured records created through one directory/data-form feature.
2. Redirects, indexes, navigation fragments, marketing repetitions, policies, and runtime composition pages are retained in \`source-coverage.json\`; relevant pages are attached to canonical feature specs as supporting evidence.
3. Every normative source extract retains its exact corpus page, original line numbers, and complete-file SHA-256.
4. Documentation status matters. \`invocation-only\`, \`high-level-documentation\`, and \`partially-documented\` specs identify real features but do not authorize invented behavior.
5. This snapshot is a specification-discovery input. When a reproducible live Wikidot observation conflicts with it, record both and implement live behavior.

## Regeneration

\`\`\`bash
node scripts/generate-wikidot-specifications.mjs
node scripts/generate-wikidot-specifications.mjs --check
\`\`\`

Set \`WIKIDOT_DOCUMENTATION_CORPUS\` only when regenerating from a different checkout of the same corpus layout.
`;

const implementationPrompt = `# Prompt: implement the complete Wikidot feature catalog with TDD

Here is the specification set. Implement every feature listed in \`docs/wikidot-specifications/catalog.json\` using test-driven development. Treat the catalog as one coherent compatibility campaign rather than creating one pull request per feature.

## Inputs

1. Read the repository's \`AGENTS.md\` completely.
2. Read \`docs/wikidot-specifications/catalog.json\`. It is the complete work queue; do not substitute a hand-selected subset.
3. Read \`docs/wikidot-specifications/CATALOG.md\` and \`docs/wikidot-specifications/README.md\`.
4. For each catalog item, read the exact Markdown file named by its \`specification\` field before designing or changing code.
5. Use \`docs/wikidot-specifications/source-coverage.json\` to inspect corroborating, redirect, runtime-composition, and non-feature source classifications when provenance is relevant.
6. Follow the repository architecture boundaries: FTML owns syntax parsing and rendering primitives; Wikijump/Deepwell owns site, page, query, import, file, permission, actor, module evaluation, and URL state; Framerail owns HTTP and browser runtime behavior.

## Authority and ambiguity

- Implement the documented contract, including legacy names, aliases, defaults, limits, output structure, URLs, permissions, side effects, and stated limitations.
- Do not modernize compatibility-sensitive syntax, DOM, identifiers, or routes.
- Live Wikidot is the behavioral oracle when the snapshot is ambiguous, incomplete, contradictory, or wrong.
- For an \`invocation-only\`, \`high-level-documentation\`, or \`partially-documented\` item, do not invent missing semantics. Design a minimal live-oracle experiment, preserve the evidence and exact fixture, update the specification, and then implement the observed behavior.
- Unsupported or unverified input must fail closed, remain literal, or use an evidenced fallback. It must not silently broaden queries or permissions.

## Mandatory TDD process

Before writing any test, produce a seam map for the current vertical slice and obtain confirmation. State the public interface being tested and why it is the appropriate observable boundary. Suggested seams in each spec are recommendations, not pre-approval.

Then repeat this loop for every behavior in every catalog item:

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

Create a machine-readable implementation ledger keyed by every \`catalog.json\` feature ID. Each entry must record:

- status: \`pending\`, \`in_progress\`, \`implemented\`, or \`blocked\`;
- confirmed public seams;
- test files and test names;
- implementation files;
- documentation and live-oracle evidence used;
- unresolved ambiguities or blockers.

There must be exactly one ledger entry per catalog item. An item is not \`implemented\` merely because adjacent code exists; all explicit behaviors in its specification must have durable tests or a concrete, documented blocker.

Keep the work in one focused campaign and normal review sequence unless repository ownership boundaries require a deliberately coordinated FTML change. Do not split routine discoveries into one pull request per example or per catalog item.

## Validation and completion

Run focused tests during each slice, then run formatting, linting, clippy/build checks, relevant integration suites, verifier suites, and browser tests in proportion to the changed surfaces. For browser-visible behavior, capture fresh evidence against exact source, dependency, fixture, and runtime identities and check visible intermediate states as well as settled DOM.

Do not declare completion until:

- every catalog item has a terminal ledger status;
- every documented behavior has regression coverage;
- every differential or fuzz result is classified;
- no known reproducible compatibility gap lacks a fix or concrete blocker;
- generated catalog/specification validation passes;
- the normal project review and merge process has completed without force or admin merge.

A merge is not a deployment. After browser-visible changes, refresh the standing runtime and verify the served URL before reporting the behavior fixed.
`;

const expectedFiles = new Map([
  ["README.md", readme],
  ["CATALOG.md", catalogMarkdown],
  ["catalog.json", serializedCatalog],
  ["source-coverage.json", `${JSON.stringify(coverage, null, 2)}\n`],
  ["live-observations.json", `${JSON.stringify(liveObservations, null, 2)}\n`],
  [
    "implementation-ledger.json",
    `${JSON.stringify(implementationLedger, null, 2)}\n`,
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
    coverage.pages.length === pages.size,
    "Source coverage does not include every corpus page",
  );
  invariant(
    coverage.unclassified_count === 0,
    "Unclassified source pages remain",
  );
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
