import {parseFragment, serializeOuter} from "parse5";

import {sha256, visibleText} from "./syntax-differential.mjs";

export const LISTPAGES_LIVE_FIXTURE_CAPTURE_SCHEMA =
  "wikijump_listpages_compat.live_fixture_capture.v1";
export const LISTPAGES_LIVE_FIXTURE_CLASSIFICATION_SCHEMA =
  "wikijump_listpages_compat.live_fixture_classification.v1";

function attribute(node, name) {
  return node.attrs?.find((entry) => entry.name === name)?.value ?? null;
}

function classes(node) {
  return new Set((attribute(node, "class") ?? "").split(/\s+/u).filter(Boolean));
}

function hasClass(node, className) {
  return classes(node).has(className);
}

function findByClass(node, className, output = []) {
  if (hasClass(node, className)) output.push(node);
  for (const child of node.childNodes ?? []) findByClass(child, className, output);
  return output;
}

function findByTag(node, tagName, output = []) {
  if (node.tagName === tagName) output.push(node);
  for (const child of node.childNodes ?? []) findByTag(child, tagName, output);
  return output;
}

function nodeText(node) {
  return visibleText(serializeOuter(node));
}

function listBoxText(caseNode) {
  return findByClass(caseNode, "list-pages-box")
    .flatMap((box) => box.childNodes ?? [])
    .filter((child) => !hasClass(child, "pager"))
    .map((child) => child.nodeName === "#text" ? child.value : serializeOuter(child))
    .map(visibleText)
    .join("\n")
    .trim();
}

function rowsFromListText(text) {
  return Array.from(text.matchAll(/([^|]+)\|/gu), (match) => match[1].trim())
    .filter(Boolean);
}

function pageNamesFromRows(rows) {
  return [...new Set(rows.flatMap((row) =>
    Array.from(row.matchAll(/lp-campaign-\d{8}-[a-z0-9-]+/gu), (match) => match[0])
  ))];
}

function extractLinks(node) {
  return findByTag(node, "a").map((link) => ({
    href: attribute(link, "href"),
    text: nodeText(link),
    classes: [...classes(link)].sort(),
  }));
}

function extractPager(caseNode) {
  const pagers = findByClass(caseNode, "pager");
  if (pagers.length === 0) return null;
  return pagers.map((pager) => ({
    text: nodeText(pager),
    pager_no: findByClass(pager, "pager-no").map(nodeText),
    current: findByClass(pager, "current").map(nodeText),
    links: extractLinks(pager),
    html_sha256: sha256(serializeOuter(pager)),
  }));
}

function counts(values) {
  const output = {};
  for (const value of values) output[value] = (output[value] ?? 0) + 1;
  return Object.fromEntries(
    Object.entries(output).sort(([left], [right]) => left.localeCompare(right)),
  );
}

export function validateLiveFixtureCapture(value) {
  if (
    value?.schema !== LISTPAGES_LIVE_FIXTURE_CAPTURE_SCHEMA ||
    !["captured", "no-page-content"].includes(value.capture_status) ||
    value.request?.authenticated !== false ||
    value.site?.unix_name !== "sandbox-for-codex" ||
    value.site?.domain !== "sandbox-for-codex.wikidot.com" ||
    typeof value.case?.case_id !== "string" ||
    typeof value.request?.url !== "string" ||
    typeof value.raw_page_html !== "string"
  ) {
    throw new Error("ListPages live fixture capture is invalid");
  }
  if (value.raw_page_html_sha256 !== sha256(value.raw_page_html)) {
    throw new Error(`raw page HTML hash does not match: ${value.case.case_id}`);
  }
  if (value.capture_status === "captured") {
    if (
      typeof value.page_content_html !== "string" ||
      value.page_content_html_sha256 !== sha256(value.page_content_html)
    ) {
      throw new Error(`page content HTML hash does not match: ${value.case.case_id}`);
    }
  }
  return value;
}

export function extractListPagesCaseBlocks(pageContentHtml) {
  const document = parseFragment(pageContentHtml);
  return findByClass(document, "lp-case").map((node) => {
    const classTokens = [...classes(node)].sort();
    const blockClass = classTokens.find((entry) => entry !== "lp-case" && entry.startsWith("lp-"));
    const listText = listBoxText(node);
    const rows = rowsFromListText(listText);
    return {
      block_class: blockClass ?? null,
      classes: classTokens,
      visible_text: nodeText(node),
      list_text: listText,
      rows,
      page_names: pageNamesFromRows(rows),
      pager: extractPager(node),
      links: extractLinks(node),
      html_sha256: sha256(serializeOuter(node)),
    };
  });
}

export function classifyListPagesLiveFixtures(captures, plan = null) {
  const classified = captures.map((rawCapture) => {
    const capture = validateLiveFixtureCapture(rawCapture);
    const blocks = capture.capture_status === "captured"
      ? extractListPagesCaseBlocks(capture.page_content_html)
      : [];
    return {
      case_id: capture.case.case_id,
      dimensions: capture.case.dimensions ?? [],
      url: capture.request.url,
      status: capture.capture_status,
      page_content_html_sha256: capture.page_content_html_sha256 ?? null,
      blocks,
    };
  });
  const blockClasses = classified.flatMap((entry) =>
    entry.blocks.map((block) => block.block_class).filter(Boolean)
  );
  return {
    schema: LISTPAGES_LIVE_FIXTURE_CLASSIFICATION_SCHEMA,
    summary: {
      captures: classified.length,
      capture_statuses: counts(classified.map((entry) => entry.status)),
      blocks: blockClasses.length,
      block_classes: [...new Set(blockClasses)].sort(),
      block_class_counts: counts(blockClasses),
      missing_case_blocks: classified
        .filter((entry) => entry.status === "captured" && entry.blocks.length === 0)
        .map((entry) => entry.case_id),
    },
    live_environment_blockers: plan?.live_environment_blockers ?? [],
    captures: classified,
  };
}
