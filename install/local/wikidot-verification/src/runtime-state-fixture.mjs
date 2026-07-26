import {sha256} from './syntax-differential.mjs';

export const RUNTIME_STATE_FIXTURE_SCHEMA =
  'wikijump_syntax_differential.runtime_state_fixture.v1';

function assertString(value, name) {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${name} must be a non-empty string`);
  }
}

function assertSha256(value, name) {
  if (typeof value !== 'string' || !/^[0-9a-f]{64}$/u.test(value)) {
    throw new Error(`${name} must be a lowercase SHA-256`);
  }
}

function validateProvenance(value, name) {
  if (value == null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  assertString(value.source, `${name}.source`);
  if (value.wikitext_hash != null && !/^[0-9a-f]{32,64}$/u.test(value.wikitext_hash)) {
    throw new Error(`${name}.wikitext_hash is invalid`);
  }
  return value;
}

function validatePageReference(value, name) {
  if (value == null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  assertString(value.site, `${name}.site`);
  assertString(value.slug, `${name}.slug`);
  if (value.provenance != null) validateProvenance(value.provenance, `${name}.provenance`);
  return value;
}

export function validateRuntimeStateFixture(value) {
  if (value?.schema !== RUNTIME_STATE_FIXTURE_SCHEMA) {
    throw new Error('runtime state fixture schema is invalid');
  }
  if (Number.isNaN(Date.parse(value.captured_at))) {
    throw new Error('runtime state fixture captured_at is invalid');
  }
  if (
    value.capture_source == null ||
    typeof value.capture_source !== 'object' ||
    Array.isArray(value.capture_source)
  ) {
    throw new Error('runtime state fixture capture_source must be an object');
  }
  assertString(value.capture_source.kind, 'runtime state fixture capture_source.kind');
  if (value.capture_source.kind === 'frozen-live-reference') {
    assertString(value.capture_source.report, 'runtime state fixture capture_source.report');
    assertSha256(
      value.capture_source.report_sha256,
      'runtime state fixture capture_source.report_sha256',
    );
  } else if (value.capture_source.kind === 'standing-corpus') {
    assertString(
      value.capture_source.database_container,
      'runtime state fixture capture_source.database_container',
    );
  } else {
    throw new Error(`runtime state fixture capture source is unsupported: ${value.capture_source.kind}`);
  }
  for (const field of ['roots', 'unresolved_pages', 'pages', 'absent_pages', 'categories']) {
    if (!Array.isArray(value[field])) {
      throw new Error(`runtime state fixture ${field} must be an array`);
    }
  }
  const wikidotUsers = value.wikidot_users ?? [];
  if (!Array.isArray(wikidotUsers)) {
    throw new Error('runtime state fixture wikidot_users must be an array');
  }
  for (const [index, root] of value.roots.entries()) {
    assertString(root, `runtime state fixture roots[${index}]`);
  }
  for (const [index, unresolved] of value.unresolved_pages.entries()) {
    assertString(unresolved, `runtime state fixture unresolved_pages[${index}]`);
  }
  const identities = new Set();
  for (const [index, page] of value.pages.entries()) {
    const name = `runtime state fixture pages[${index}]`;
    validatePageReference(page, name);
    assertString(page.title, `${name}.title`);
    if (typeof page.wikitext !== 'string') throw new Error(`${name}.wikitext must be a string`);
    assertSha256(page.source_sha256, `${name}.source_sha256`);
    if (sha256(page.wikitext) !== page.source_sha256) {
      throw new Error(`${name} source hash does not match`);
    }
    validateProvenance(page.provenance, `${name}.provenance`);
    const identity = `${page.site}:${page.slug}`;
    if (identities.has(identity)) throw new Error(`runtime state fixture page is duplicated: ${identity}`);
    identities.add(identity);
  }
  for (const [index, page] of value.absent_pages.entries()) {
    const name = `runtime state fixture absent_pages[${index}]`;
    validatePageReference(page, name);
    const identity = `${page.site}:${page.slug}`;
    if (identities.has(identity)) {
      throw new Error(`runtime state fixture page has conflicting states: ${identity}`);
    }
    identities.add(identity);
  }
  const categories = new Set();
  for (const [index, category] of value.categories.entries()) {
    const name = `runtime state fixture categories[${index}]`;
    validatePageReference(category, name);
    if (category.oracle_id != null && !Number.isSafeInteger(category.oracle_id)) {
      throw new Error(`${name}.oracle_id must be a safe integer`);
    }
    const identity = `${category.site}:${category.slug}`;
    if (categories.has(identity)) {
      throw new Error(`runtime state fixture category is duplicated: ${identity}`);
    }
    categories.add(identity);
  }
  const userIds = new Set();
  const userNames = new Set();
  const userSlugs = new Set();
  for (const [index, user] of wikidotUsers.entries()) {
    const name = `runtime state fixture wikidot_users[${index}]`;
    if (user == null || typeof user !== 'object' || Array.isArray(user)) {
      throw new Error(`${name} must be an object`);
    }
    if (!Number.isSafeInteger(user.user_id) || user.user_id <= 0 || user.user_id > 2_147_483_647) {
      throw new Error(`${name}.user_id must be a positive 32-bit integer`);
    }
    assertString(user.name, `${name}.name`);
    assertString(user.slug, `${name}.slug`);
    if (!/^[a-z0-9-]+$/u.test(user.slug)) {
      throw new Error(`${name}.slug is invalid`);
    }
    validateProvenance(user.provenance, `${name}.provenance`);
    if (Number.isNaN(Date.parse(user.provenance.captured_at))) {
      throw new Error(`${name}.provenance.captured_at is invalid`);
    }
    assertSha256(user.provenance.capture_file_sha256, `${name}.provenance.capture_file_sha256`);
    assertSha256(user.provenance.saved_source_sha256, `${name}.provenance.saved_source_sha256`);
    assertSha256(user.provenance.wikidot_html_sha256, `${name}.provenance.wikidot_html_sha256`);
    if (
      !Number.isSafeInteger(user.provenance.capture_line) ||
      user.provenance.capture_line <= 0 ||
      !Number.isSafeInteger(user.provenance.page_identity) ||
      user.provenance.page_identity <= 0
    ) {
      throw new Error(`${name}.provenance capture identity is invalid`);
    }
    if (userIds.has(user.user_id) || userNames.has(user.name) || userSlugs.has(user.slug)) {
      throw new Error(`${name} identity is duplicated`);
    }
    userIds.add(user.user_id);
    userNames.add(user.name);
    userSlugs.add(user.slug);
  }
  return {...value, wikidot_users: wikidotUsers};
}

export function validateRuntimeStateFixtureInput(value) {
  if (
    value == null ||
    typeof value.path !== 'string' ||
    value.path.length === 0
  ) {
    throw new Error('runtime state fixture input path is invalid');
  }
  assertSha256(value.sha256, `runtime state fixture input hash for ${value.path}`);
  return {
    path: value.path,
    sha256: value.sha256,
    fixture: validateRuntimeStateFixture(value.fixture),
  };
}
