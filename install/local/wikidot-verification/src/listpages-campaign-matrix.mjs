import fs from "node:fs/promises";
import path from "node:path";

export const LISTPAGES_MATRIX_SCHEMA =
  "wikijump_listpages_compat.differential_matrix.v1";

function jsonLine(record) {
  return `${JSON.stringify(record)}\n`;
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

async function readJsonl(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  if (!text.trim()) return [];
  return text
    .trimEnd()
    .split(/\r?\n/u)
    .map((line) => JSON.parse(line));
}

function firstByCluster(invocations) {
  const rows = new Map();
  for (const invocation of invocations) {
    if (!rows.has(invocation.semantic_cluster_key)) {
      rows.set(invocation.semantic_cluster_key, invocation);
    }
  }
  return rows;
}

function authoredSource(invocation) {
  if (!invocation.balanced) return invocation.head;
  return `${invocation.head}${invocation.body}[[/module]]`;
}

function variablesFromSource(source) {
  return [...source.matchAll(/%%([^%\r\n]+)%%/gu)]
    .map((match) => match[1].trim())
    .filter((value, index, values) => value && values.indexOf(value) === index);
}

function caseId(prefix, index, label) {
  return `${prefix}-${String(index + 1).padStart(4, "0")}-${label
    .replace(/[^a-z0-9]+/giu, "-")
    .replace(/^-|-$/gu, "")
    .toLowerCase()}`;
}

function generatedCase(
  index,
  label,
  source,
  dimensions,
  documentationClaimIds = [],
) {
  return {
    schema: `${LISTPAGES_MATRIX_SCHEMA}.case`,
    id: caseId("lpgen", index, label),
    origin: "generated",
    label,
    source,
    dimensions,
    documentation_claim_ids: documentationClaimIds,
    template_variables: variablesFromSource(source),
    verification_status: "pending-live",
    expected_live_behavior: null,
    local_behavior: null,
    classification: "unverified",
  };
}

function listModule(attrs, body = "%%title_linked%%") {
  const head = Object.entries(attrs)
    .map(([name, value]) => `${name}="${value}"`)
    .join(" ");
  return `[[module ListPages${head ? ` ${head}` : ""}]]\n${body}\n[[/module]]`;
}

function findClaimIds(claims, pattern, limit = 6) {
  const regex = pattern instanceof RegExp ? pattern : new RegExp(pattern, "iu");
  const priority = (claim) => {
    const fullname = claim.source?.page_fullname ?? "";
    if (fullname === "doc-include:page-selection") return 0;
    if (fullname === "doc-modules:listpages-module") return 1;
    if (fullname === "doc-include:listpages-module-prev") return 2;
    if (fullname.startsWith("doc-modules:")) return 3;
    return 10;
  };
  return claims
    .filter((claim) => regex.test(claim.claim))
    .sort(
      (left, right) =>
        priority(left) - priority(right) || left.id.localeCompare(right.id),
    )
    .slice(0, limit)
    .map((claim) => claim.id);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function findHashMagicClaimIds(claims, hash) {
  const hashes = hash === "#_history/p/2" ? ["#_history"] : [hash];
  const regexes = hashes.map((value) => new RegExp(escapeRegExp(value), "iu"));
  return claims
    .filter((claim) => regexes.some((regex) => regex.test(claim.claim)))
    .sort((left, right) => {
      const leftFullname = left.source?.page_fullname ?? "";
      const rightFullname = right.source?.page_fullname ?? "";
      const priority = (fullname) => {
        if (fullname === "doc-wiki-syntax:links") return 0;
        if (fullname === "nav:doc-wiki-syntax") return 1;
        return 10;
      };
      return (
        priority(leftFullname) - priority(rightFullname) ||
        left.id.localeCompare(right.id)
      );
    })
    .slice(0, 6)
    .map((claim) => claim.id);
}

function buildGeneratedCases(claims) {
  const rows = [];
  const add = (label, source, dimensions, claimPattern = null) => {
    rows.push(
      generatedCase(
        rows.length,
        label,
        source,
        dimensions,
        claimPattern ? findClaimIds(claims, claimPattern) : [],
      ),
    );
  };

  for (const [name, values] of Object.entries({
    category: [
      "",
      ".",
      "*",
      "fragment",
      "component theme",
      "-fragment",
      "scp,fragment",
    ],
    tags: [
      "",
      "-",
      "=",
      "==",
      "+scp -_hidden",
      "scp tale,+featured",
      "+apple,-banana",
    ],
    parent: ["", "-", "=", "-=", ".", "system:start"],
    range: ["", ".", "before", "after", "others", "bogus"],
    pagetype: ["normal", "hidden", "*", "", "bogus"],
    name: ["", "=", "scp-*", "scp-%", "literal page"],
    fullname: ["", "component:wikimodule", "="],
  })) {
    for (const value of values) {
      add(
        `${name} selector ${value || "empty"}`,
        listModule({ [name]: value }),
        ["selector", name, value === "" ? "empty-value" : "value"],
        new RegExp(`\\b${name}\\b`, "iu"),
      );
    }
  }

  for (const [name, values] of Object.entries({
    created_at: [
      "=",
      "2024",
      "2024.05",
      ">2024.05",
      "<=2024.05.10",
      "<>2024",
      "last hour",
      "older than 2 weeks",
      "not-a-date",
    ],
    updated_at: ["2024", "last 3 day", "older than month", "bad"],
    rating: ["0", "=", ">0", "<=-1", "<>5", "bad"],
    votes: ["0", "=", ">10", "<>2", "bad"],
  })) {
    for (const value of values) {
      add(
        `${name} selector ${value}`,
        listModule({ [name]: value }),
        ["selector", name, "comparison-or-date"],
        new RegExp(
          `\\b${name}\\b|Rating selector|Votes selector|date selector`,
          "iu",
        ),
      );
    }
  }

  for (const order of [
    "name",
    "fullname",
    "title",
    "created_by",
    "created_at desc",
    "created_at desc desc",
    "created_at asc",
    "updated_at desc",
    "size desc",
    "rating desc",
    "votes desc",
    "revisions desc",
    "comments desc",
    "random",
    "_mainword",
    "_albums::integer desc",
    "dateCreatedAsc",
    "dateCreatedDesc",
    "pageLengthDesc",
    "unknown",
    "",
  ]) {
    add(
      `order ${order || "empty"}`,
      listModule({ order }),
      ["ordering", order === "" ? "empty-value" : "value"],
      /order|Order criteria|dateCreated|pageLength/iu,
    );
  }

  for (const [name, values] of Object.entries({
    limit: [
      "",
      "0",
      "1",
      "20",
      "250",
      "251",
      "-1",
      "2.5",
      "999999999",
      "@URL",
      "@URL|0",
    ],
    perPage: [
      "",
      "0",
      "1",
      "20",
      "250",
      "251",
      "-1",
      "2.5",
      "999999999",
      "@URL",
      "@URL|20",
    ],
    offset: ["", "0", "1", "-1", "2.5", "999999999", "@URL", "@URL|0"],
  })) {
    for (const value of values) {
      add(
        `${name} ${value || "empty"}`,
        listModule({ [name]: value }),
        [
          "pagination",
          name,
          value.startsWith("@URL") ? "url-driven" : "literal",
        ],
        new RegExp(`\\b${name}\\b|Pagination`, "iu"),
      );
    }
  }

  for (const attrs of [
    { separate: "yes", wrapper: "yes" },
    { separate: "no", wrapper: "yes" },
    { separate: "no", wrapper: "no" },
    { separate: "true", wrapper: "false" },
    { separate: "", wrapper: "" },
  ]) {
    add(
      `body containers ${attrs.separate || "empty"} ${attrs.wrapper || "empty"}`,
      listModule(attrs, "%%index%%. %%title%%"),
      ["rendering", "containers"],
      /separate|wrapper|container/iu,
    );
  }

  add(
    "prepend append with separate no",
    listModule(
      { separate: "no", prependLine: "HEAD", appendLine: "FOOT" },
      "* %%title_linked%%",
    ),
    ["rendering", "prependLine", "appendLine"],
    /prependLine|appendLine|Header specifier|Footer specifier/iu,
  );
  add(
    "head body foot sections",
    listModule(
      { separate: "no", wrapper: "no" },
      "[[head]]||~ Title ||[[/head]]\n[[body]]|| %%title_linked%% ||[[/body]]\n[[foot]]|| footer ||[[/foot]]",
    ),
    ["rendering", "sections"],
    /\[\[head\]\]|\[\[body\]\]|\[\[foot\]\]/iu,
  );
  add(
    "code tag in module body limitation",
    listModule({}, "[[code]]\n%%title%%\n[[/code]]"),
    ["malformed", "body-parser-limitation"],
    /Module body cannot contain/iu,
  );
  add(
    "html tag in module body limitation",
    listModule({}, "[[html]]<b>%%title%%</b>[[/html]]"),
    ["malformed", "body-parser-limitation"],
    /Module body cannot contain/iu,
  );

  for (const attrs of [
    { rss: "Feed Title" },
    {
      rss: "Feed Title",
      rssDescription: "Description",
      rssHome: "blog:_start",
      rssLimit: "3",
    },
    { rss: "Feed Title", rssOnly: "yes" },
    { rssTitle: "Old Feed Title" },
  ]) {
    add(
      "rss " + Object.keys(attrs).join(" "),
      listModule(attrs),
      ["rss"],
      /RSS|rss/iu,
    );
  }

  for (const source of [
    '[[module ListPages tags="+scp" tags="-tale"]]\n%%title%%\n[[/module]]',
    '[[module ListPages order="title" order="rating desc"]]\n%%title%%\n[[/module]]',
    '[[module ListPages range="." range="others"]]\n%%title%%\n[[/module]]',
    '[[module ListPages limit="5" perPage="2" reverse="yes"]]\n%%index%% %%title%%\n[[/module]]',
    '[[module ListPages category="*" category="-fragment" tags="+scp" tags="-archived"]]\n%%title%%\n[[/module]]',
  ]) {
    add(
      "duplicate conflicting attributes",
      source,
      ["duplicates", "conflicts"],
      /deprecated|range|tags|order/iu,
    );
  }

  for (const source of [
    '[[module\tListPages\tcategory="."   tags="+scp"]]\n%%title%%\n[[/module]]',
    '[[module ListPages category = "." tags = "+scp" ]]\n%%title%%\n[[/module]]',
    "[[module ListPages category=. tags=+scp]]\n%%title%%\n[[/module]]",
    "[[module ListPages category='.']]\n%%title%%\n[[/module]]",
    '[[module ListPages category="fragment" parent="." order="name"" limit="1" offset="@URL|0"]]\n%%content%%\n[[/module]]',
    '[[module ListPages tags="+scp rating="<0" separate="no"]]\n%%title%%\n[[/module]]',
    '[[module ListPages category="fragment"]]\n[[module ListPages range="."]]%%content%%[[/module]]\n[[/module]]',
    '[[module ListPages category="fragment"]]\n%%title%%',
    '[[module ListPages category="fragment"\n%%title%%',
  ]) {
    add(
      "syntax whitespace malformed",
      source,
      ["syntax", "whitespace-or-malformed"],
      /syntax|Module body|default/iu,
    );
  }

  return rows;
}

function buildNavigationCases(claims) {
  const claimIds = findClaimIds(
    claims,
    /@URL|pagination|urlAttrPrefix|\/p\//iu,
    12,
  );
  const urls = [
    "",
    "/p/1",
    "/p/2",
    "/p/3",
    "/p/0",
    "/p/-1",
    "/p/abc",
    "/p/2.5",
    "/p/999999999",
    "/tag/alpha/p/2",
    "/p/2/tag/alpha",
    "/p/2/p/3",
    "/category/fragment/p/2",
    "/offset/1/p/2",
    "/prefix_p/2",
    "/page2_limit/1/page3_limit/2",
    "?q=1",
    "/p/2?q=1",
    "/p/2#fragment",
  ];
  return urls.map((url, index) => ({
    schema: `${LISTPAGES_MATRIX_SCHEMA}.navigation_case`,
    id: caseId("lpnav", index, url || "root"),
    origin: "generated-navigation",
    url_suffix: url,
    interactions: [
      "initial-load",
      "direct-load",
      "reload",
      "back-forward",
      "client-navigation",
      "full-document-navigation",
    ],
    module_shapes: [
      listModule({ perPage: "2", limit: "5", order: "title" }),
      listModule({
        perPage: "2",
        limit: "5",
        order: "title",
        urlAttrPrefix: "page2",
      }),
    ],
    documentation_claim_ids: claimIds,
    verification_status: "pending-live",
    classification: "unverified",
  }));
}

function buildHashMagicAuditCases(claims) {
  const documented = [
    ["#_wantedpages", "lists Wanted Pages"],
    ["#_orphanedpages", "lists Orphaned Pages"],
    ["#_draftpages", "lists Draft Pages on site"],
    ["#_editpage", "opens Editor"],
    ["#_edittags", "opens Tag Editor"],
    ["#_history", "displays History"],
    ["#_files", "lists Files attached to the page"],
    ["#_sitetools", "opens Site Tools"],
  ].map(([hash, documentedBehavior]) => ({
    hash,
    documented: true,
    documented_behavior: documentedBehavior,
    provenance: [
      {
        kind: "local-corpus",
        page_fullname: "doc-wiki-syntax:links",
        path: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:links/source.wikidot.txt",
        lines: "82-94",
      },
      {
        kind: "external-reference-absent-from-local-corpus",
        url: "http://community.wikidot.com/howto:use-hash-magic-urls-for-option-buttons",
        note: "Referenced by the campaign request; fetched only because no corresponding local corpus page was present.",
      },
    ],
  }));
  const experimental = [
    ["#_tags", "undocumented UI alias probe"],
    ["#_discuss", "undocumented UI alias probe"],
    ["#_edit", "undocumented UI alias probe"],
    ["#_page-options", "undocumented UI alias probe"],
    [
      "#_history/p/2",
      "composition probe: hash-magic history anchor with a pagination-looking suffix",
    ],
  ].map(([hash, note]) => ({
    hash,
    documented: false,
    documented_behavior: null,
    provenance: [
      {
        kind: "campaign-generated-probe",
        note,
      },
    ],
  }));
  return [...documented, ...experimental].map((target, index) => ({
    schema: `${LISTPAGES_MATRIX_SCHEMA}.hash_magic_case`,
    id: caseId("lphash", index, target.hash),
    origin: "generated-hash-magic-audit",
    hash: target.hash,
    scope: "inventory-only-unless-listpages-affected",
    documented: target.documented,
    documented_behavior: target.documented_behavior,
    provenance: target.provenance,
    documentation_claim_ids: target.documented
      ? findHashMagicClaimIds(claims, target.hash)
      : target.hash === "#_history/p/2"
        ? findHashMagicClaimIds(claims, target.hash)
        : [],
    verification_status: "pending-live",
    classification: "unverified",
  }));
}

export async function buildListPagesMatrix({ inventoryDir }) {
  const [claims, invocations, clustersFile] = await Promise.all([
    readJsonl(path.join(inventoryDir, "documentation-claims.jsonl")),
    readJsonl(path.join(inventoryDir, "corpus-listpages-invocations.jsonl")),
    readJson(path.join(inventoryDir, "corpus-listpages-clusters.json")),
  ]);

  const firstInvocations = firstByCluster(invocations);
  const clusterCases = clustersFile.clusters.map((cluster, index) => {
    const invocation = firstInvocations.get(cluster.semantic_cluster_key);
    return {
      schema: `${LISTPAGES_MATRIX_SCHEMA}.case`,
      id: caseId("lpcorpus", index, cluster.semantic_cluster_key.slice(0, 12)),
      origin: "corpus-cluster-representative",
      semantic_cluster_key: cluster.semantic_cluster_key,
      corpus_occurrence_count: cluster.count,
      source: authoredSource(invocation),
      dimensions: [
        "corpus",
        ...cluster.argument_signature.map((arg) => `arg:${arg.split("=")[0]}`),
        ...cluster.template_variables.map((variable) => `var:${variable}`),
        ...cluster.body_sections.map((section) => `section:${section}`),
      ],
      provenance: cluster.first_provenance,
      verification_status: "pending-live",
      expected_live_behavior: null,
      local_behavior: null,
      classification: "unverified",
    };
  });

  const invocationCases = invocations.map((invocation) => ({
    schema: `${LISTPAGES_MATRIX_SCHEMA}.corpus_invocation_case`,
    id: invocation.id,
    origin: "corpus-invocation",
    semantic_cluster_key: invocation.semantic_cluster_key,
    source_sha256: invocation.source_sha256,
    provenance: {
      branch: invocation.branch,
      page_fullname: invocation.page_fullname,
      source_path: invocation.source_path,
      line_start: invocation.line_start,
      line_end: invocation.line_end,
    },
    balanced: invocation.balanced,
    malformed_reason: invocation.malformed_reason,
    dimensions: [
      "corpus",
      ...invocation.attributes.map((attr) => `arg:${attr.name.toLowerCase()}`),
      ...invocation.duplicate_attributes.map((attr) => `duplicate:${attr}`),
      ...invocation.url_driven_attributes.map((attr) => `url:${attr}`),
      ...invocation.template_variables.map((variable) => `var:${variable}`),
      ...invocation.body_sections.map((section) => `section:${section}`),
    ],
    verification_status: "pending-live",
    classification: "unverified",
  }));

  const generatedCases = buildGeneratedCases(claims);
  const navigationCases = buildNavigationCases(claims);
  const hashMagicCases = buildHashMagicAuditCases(claims);

  return {
    schema: LISTPAGES_MATRIX_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      inventory_dir: inventoryDir,
      documentation_claim_count: claims.length,
      corpus_invocation_count: invocations.length,
      corpus_cluster_count: clustersFile.clusters.length,
    },
    corpus_cluster_cases: clusterCases,
    corpus_invocation_cases: invocationCases,
    generated_cases: generatedCases,
    navigation_cases: navigationCases,
    hash_magic_cases: hashMagicCases,
    summary: {
      corpus_cluster_case_count: clusterCases.length,
      corpus_invocation_case_count: invocationCases.length,
      generated_case_count: generatedCases.length,
      navigation_case_count: navigationCases.length,
      hash_magic_case_count: hashMagicCases.length,
      pending_live_count:
        clusterCases.length +
        invocationCases.length +
        generatedCases.length +
        navigationCases.length +
        hashMagicCases.length,
    },
  };
}

export async function writeListPagesMatrix(matrix, outputDir) {
  await fs.mkdir(outputDir, { recursive: true, mode: 0o700 });
  await Promise.all([
    fs.writeFile(
      path.join(outputDir, "matrix-summary.json"),
      `${JSON.stringify(
        {
          schema: matrix.schema,
          generated_at: matrix.generated_at,
          inputs: matrix.inputs,
          summary: matrix.summary,
          files: {
            corpus_cluster_cases: "corpus-cluster-cases.jsonl",
            corpus_invocation_cases: "corpus-invocation-cases.jsonl",
            generated_cases: "generated-listpages-cases.jsonl",
            navigation_cases: "navigation-cases.jsonl",
            hash_magic_cases: "hash-magic-audit-cases.jsonl",
          },
        },
        null,
        2,
      )}\n`,
      { mode: 0o600 },
    ),
    fs.writeFile(
      path.join(outputDir, "corpus-cluster-cases.jsonl"),
      matrix.corpus_cluster_cases.map(jsonLine).join(""),
      { mode: 0o600 },
    ),
    fs.writeFile(
      path.join(outputDir, "corpus-invocation-cases.jsonl"),
      matrix.corpus_invocation_cases.map(jsonLine).join(""),
      { mode: 0o600 },
    ),
    fs.writeFile(
      path.join(outputDir, "generated-listpages-cases.jsonl"),
      matrix.generated_cases.map(jsonLine).join(""),
      { mode: 0o600 },
    ),
    fs.writeFile(
      path.join(outputDir, "navigation-cases.jsonl"),
      matrix.navigation_cases.map(jsonLine).join(""),
      { mode: 0o600 },
    ),
    fs.writeFile(
      path.join(outputDir, "hash-magic-audit-cases.jsonl"),
      matrix.hash_magic_cases.map(jsonLine).join(""),
      { mode: 0o600 },
    ),
  ]);
}
