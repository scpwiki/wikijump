import {parse, serializeOuter} from "parse5";

import {canonicalDom, sha256, visibleText} from "./syntax-differential.mjs";

export const SAVED_PAGE_REFERENCE_SCHEMA =
  "wikijump_syntax_differential.wikidot_saved_page_reference.v1";
export const RUNTIME_IDENTITY_SCHEMA =
  "wikijump_syntax_differential.wikijump_runtime_identity.v1";

function classes(node) {
  return new Set(
    (node.attrs?.find((attribute) => attribute.name === "class")?.value ?? "")
      .split(/\s+/u)
      .filter(Boolean),
  );
}

function findByClass(node, className, output = []) {
  if (classes(node).has(className)) output.push(node);
  for (const child of node.childNodes ?? []) findByClass(child, className, output);
  return output;
}

function validateSha(value, name, length = 64) {
  if (typeof value !== "string" || !new RegExp(`^[0-9a-f]{${length}}$`, "u").test(value)) {
    throw new Error(`${name} is invalid`);
  }
}

export function validateRuntimeIdentity(identity) {
  if (identity?.schema !== RUNTIME_IDENTITY_SCHEMA) {
    throw new Error("Wikijump runtime identity schema is unsupported");
  }
  validateSha(identity.wikijump_sha, "runtime Wikijump SHA", 40);
  validateSha(identity.ftml_sha, "runtime FTML SHA", 40);
  validateSha(identity.dependency_lock_sha256, "runtime dependency lock SHA");
  validateSha(identity.executable_sha256, "runtime executable SHA");
  validateSha(identity.runtime_config_sha256, "runtime configuration SHA");
  return identity;
}

export function validateSavedPageReference(reference) {
  if (reference?.schema !== SAVED_PAGE_REFERENCE_SCHEMA) {
    throw new Error("Wikidot saved-page reference schema is unsupported");
  }
  if (reference.actor?.authenticated !== false || reference.provenance?.mutated !== false) {
    throw new Error("Wikidot saved-page reference must be an anonymous read");
  }
  const selector = reference.case?.selector;
  if (typeof selector !== "string" || !/^\.[A-Za-z0-9_-]+$/u.test(selector)) {
    throw new Error("saved-page selector must be one class selector");
  }
  validateSha(reference.page?.source_sha256, "Wikidot source SHA");
  validateSha(reference.selected_html_sha256, "Wikidot selected HTML SHA");
  if (reference.selected_html_sha256 !== sha256(reference.selected_html ?? "")) {
    throw new Error("Wikidot selected HTML does not match its identity");
  }
  return reference;
}

export function extractSelectedHtml(documentHtml, selector) {
  const className = selector.slice(1);
  const selected = findByClass(parse(documentHtml), className);
  if (selected.length !== 1) {
    throw new Error(`local selector ${selector} returned ${selected.length} nodes`);
  }
  return serializeOuter(selected[0]);
}

function literalCheck(html, forbiddenLiterals) {
  const found = forbiddenLiterals.filter((literal) => html.includes(literal));
  return {status: found.length === 0 ? "match" : "mismatch", found};
}

function classCheck(html, requiredTokens) {
  const document = parse(html);
  const matching = findByClass(document, requiredTokens[0]).some((node) => {
    const tokens = classes(node);
    return requiredTokens.every((token) => tokens.has(token));
  });
  return {status: matching ? "match" : "mismatch", required_tokens: requiredTokens};
}

export function compareSavedPageRuntime(reference, localDocumentHtml, runtimeIdentity) {
  validateSavedPageReference(reference);
  validateRuntimeIdentity(runtimeIdentity);
  const localHtml = extractSelectedHtml(localDocumentHtml, reference.case.selector);
  const wikidotHtml = reference.selected_html;
  const wikidotDom = canonicalDom(wikidotHtml);
  const wikijumpDom = canonicalDom(localHtml);
  const domMatches = JSON.stringify(wikidotDom) === JSON.stringify(wikijumpDom);
  const wikidotText = visibleText(wikidotHtml);
  const wikijumpText = visibleText(localHtml);
  const textMatches = wikidotText === wikijumpText;
  const expected = reference.case.expected;
  const classShape = classCheck(localHtml, expected.required_class_tokens);
  const unexpandedDirectives = literalCheck(localHtml, expected.forbidden_literals);
  const checks = {
    dom_hierarchy_child_order_and_attributes: {
      status: domMatches ? "match" : "mismatch",
    },
    visible_text: {
      status: textMatches ? "match" : "mismatch",
      wikidot: wikidotText,
      wikijump: wikijumpText,
    },
    required_class_shape: classShape,
    unexpanded_directives: unexpandedDirectives,
  };
  const status = Object.values(checks).every((check) => check.status === "match")
    ? "match"
    : "mismatch";
  return {
    schema: "wikijump_syntax_differential.saved_page_runtime_comparison.v1",
    case_id: reference.case.case_id,
    status,
    checks,
    identities: {
      wikidot: {
        site: reference.site,
        page: reference.page,
        selected_html_sha256: reference.selected_html_sha256,
      },
      wikijump: {
        ...runtimeIdentity,
        selected_html_sha256: sha256(localHtml),
      },
    },
    ...(status === "match"
      ? {}
      : {diagnostic: {wikidot_html: wikidotHtml, wikijump_html: localHtml}}),
  };
}
