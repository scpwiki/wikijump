import path from "node:path";

import {
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
} from "../../local/wikidot-verification/src/standing-browser-parity-util.mjs";

export const STANDING_REFRESH_KIND = "standing-promotion";
const SERVICES = Object.freeze(["deepwell", "framerail", "wws"]);
const LEGACY_PROTECTED_VOLUMES = Object.freeze([
  "runtime50x-postgres-data",
  "runtime50x-files-data",
]);
const PROTECTED_VOLUMES = Object.freeze([
  "wikijump-standing-postgres-data",
  "wikijump-standing-files-data",
  "wikijump-standing-cache-data",
]);
const GIT_OBJECT = /^[0-9a-f]{40}$/u;
const IMAGE_ID = /^sha256:[0-9a-f]{64}$/u;

function exactKeys(value, expected, name) {
  requirePlainObject(value, name);
  if (
    JSON.stringify(Object.keys(value).sort()) !==
    JSON.stringify([...expected].sort())
  ) {
    throw new Error(`${name} has missing or unknown fields`);
  }
}

function requireGitObject(value, name) {
  if (!GIT_OBJECT.test(value ?? "")) throw new Error(`${name} must be a Git object id`);
  return value;
}

function requireImageId(value, name) {
  if (!IMAGE_ID.test(value ?? "")) throw new Error(`${name} must be an immutable image id`);
  return value;
}

function requirePathDigest(value, name) {
  exactKeys(value, ["path", "sha256"], name);
  if (!path.isAbsolute(value.path)) throw new Error(`${name}.path must be absolute`);
  requireSha256(value.sha256, `${name}.sha256`);
  return value;
}

function requireDuration(value, name) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new Error(`${name} must be a non-negative duration`);
  }
}

function validateImages(value, name) {
  exactKeys(value, SERVICES, name);
  for (const service of SERVICES) {
    const image = value[service];
    exactKeys(image, ["reference", "id", "repo_digests", "labels"], `${name}.${service}`);
    requireImageId(image.id, `${name}.${service}.id`);
    if (image.reference !== image.id) throw new Error(`${name}.${service}.reference is mutable`);
    if (!Array.isArray(image.repo_digests) || image.repo_digests.some((digest) => typeof digest !== "string")) {
      throw new Error(`${name}.${service}.repo_digests is invalid`);
    }
    requirePlainObject(image.labels, `${name}.${service}.labels`);
  }
  return value;
}

function validateResourceImages(value, name, images) {
  exactKeys(value, SERVICES, name);
  for (const service of SERVICES) {
    const resource = value[service];
    exactKeys(resource, ["owner", "keep_until", "id"], `${name}.${service}`);
    requireNonEmptyString(resource.owner, `${name}.${service}.owner`);
    requireNonEmptyString(resource.keep_until, `${name}.${service}.keep_until`);
    requireImageId(resource.id, `${name}.${service}.id`);
    if (resource.id !== images[service].id) {
      throw new Error(`${name}.${service}.id does not match its image identity`);
    }
  }
}

function validateRefreshRuntimeIdentity(value, refresh) {
  exactKeys(value, ["path", "sha256", "identity"], "standing refresh runtime identity");
  requirePathDigest({path: value.path, sha256: value.sha256}, "standing refresh runtime identity");
  exactKeys(
    value.identity,
    ["schema", "wikijump_sha", "ftml_sha", "dependency_lock_sha256", "executable_sha256", "runtime_config_sha256"],
    "standing refresh runtime identity.identity",
  );
  if (value.identity.schema !== "wikijump_syntax_differential.wikijump_runtime_identity.v1") {
    throw new Error("standing refresh runtime identity has an unsupported schema");
  }
  requireGitObject(value.identity.wikijump_sha, "standing refresh runtime identity Wikijump commit");
  requireGitObject(value.identity.ftml_sha, "standing refresh runtime identity FTML commit");
  for (const field of ["dependency_lock_sha256", "executable_sha256", "runtime_config_sha256"]) {
    requireSha256(value.identity[field], `standing refresh runtime identity.${field}`);
  }
  if (value.identity.wikijump_sha !== refresh.wikijump_sha || value.identity.ftml_sha !== refresh.ftml_sha) {
    throw new Error("standing refresh runtime identity source is stale");
  }
  if (value.identity.executable_sha256 !== refresh.images.deepwell.id.slice("sha256:".length)) {
    throw new Error("standing refresh runtime identity executable is stale");
  }
  return value;
}

export function validateStandingRefreshReceipt(value) {
  exactKeys(
    value,
    [
      "schema_version",
      "kind",
      "status",
      "run_id",
      "started_at",
      "completed_at",
      "activation_duration_seconds",
      "image_verification_duration_seconds",
      "compose_activation_duration_seconds",
      "health_duration_seconds",
      "canary_duration_seconds",
      "wikijump_sha",
      "wikijump_tree",
      "ftml_sha",
      "dependency_lock_sha256",
      "promotion_precondition",
      "runtime_home",
      "prepared_receipt",
      "project_name",
      "network_name",
      "images",
      "rollback_images",
      "protected_volumes",
      "saved_page_render_freshness",
      "runtime_differential_identity",
      "health",
      "canary",
      "cleanup",
      "resource_disposition",
      ...(value?.schema_version === 3 ? ["render_ahead"] : []),
    ],
    "standing refresh receipt",
  );
  if (![1, 2, 3].includes(value.schema_version) || value.kind !== STANDING_REFRESH_KIND || value.status !== "pass") {
    throw new Error("standing refresh receipt is not a passing canonical receipt");
  }
  requireNonEmptyString(value.run_id, "standing refresh.run_id");
  requireNonEmptyString(value.started_at, "standing refresh.started_at");
  requireNonEmptyString(value.completed_at, "standing refresh.completed_at");
  for (const field of [
    "activation_duration_seconds",
    "image_verification_duration_seconds",
    "compose_activation_duration_seconds",
    "health_duration_seconds",
    "canary_duration_seconds",
  ]) requireDuration(value[field], `standing refresh.${field}`);
  requireGitObject(value.wikijump_sha, "standing refresh Wikijump commit");
  requireGitObject(value.wikijump_tree, "standing refresh Wikijump tree");
  requireGitObject(value.ftml_sha, "standing refresh FTML commit");
  requireSha256(value.dependency_lock_sha256, "standing refresh dependency lock");
  requirePathDigest(value.promotion_precondition, "standing refresh promotion precondition");
  requirePathDigest(value.prepared_receipt, "standing refresh prepared receipt");
  requireNonEmptyString(value.runtime_home, "standing refresh.runtime_home");
  requireNonEmptyString(value.project_name, "standing refresh.project_name");
  if (value.project_name !== "wikijump-standing") throw new Error("standing refresh project is not standing");
  requireNonEmptyString(value.network_name, "standing refresh.network_name");
  const protectedVolumes = value.schema_version === 1
    ? LEGACY_PROTECTED_VOLUMES
    : PROTECTED_VOLUMES;
  if (JSON.stringify(value.protected_volumes) !== JSON.stringify(protectedVolumes)) {
    throw new Error("standing refresh protected volumes are not canonical");
  }

  exactKeys(
    value.saved_page_render_freshness,
    ["expected_compiled_generator", "stale_pages_remaining", "status"],
    "standing refresh saved page render freshness",
  );
  requireNonEmptyString(
    value.saved_page_render_freshness.expected_compiled_generator,
    "standing refresh saved page render freshness.expected_compiled_generator",
  );
  if (
    value.saved_page_render_freshness.status !== "pass" ||
    !Number.isSafeInteger(value.saved_page_render_freshness.stale_pages_remaining) ||
    value.saved_page_render_freshness.stale_pages_remaining !== 0
  ) {
    throw new Error("standing refresh saved page render freshness is not passing");
  }

  if (value.schema_version === 3) {
    const ahead = value.render_ahead;
    if (ahead?.status !== "pass" && ahead?.status !== "not-needed") {
      throw new Error("standing refresh has no passing render-ahead decision");
    }
    exactKeys(
      ahead,
      ahead.status === "pass"
        ? ["status", "expected_compiled_generator", "stale_pages_before", "receipt"]
        : ["status", "expected_compiled_generator", "stale_pages_before"],
      "standing refresh render-ahead",
    );
    if (
      ahead.expected_compiled_generator !== value.saved_page_render_freshness.expected_compiled_generator ||
      !Number.isSafeInteger(ahead.stale_pages_before) || ahead.stale_pages_before < 0 ||
      (ahead.status === "not-needed" && ahead.stale_pages_before !== 0) ||
      (ahead.status === "pass" && ahead.stale_pages_before === 0)
    ) {
      throw new Error("standing refresh render-ahead is inconsistent with saved-page freshness");
    }
    if (ahead.status === "pass") {
      requirePathDigest(ahead.receipt, "standing refresh render-ahead receipt");
    }
  }

  validateImages(value.images, "standing refresh.images");
  validateImages(value.rollback_images, "standing refresh.rollback_images");
  exactKeys(value.health, SERVICES, "standing refresh.health");
  for (const service of SERVICES) if (value.health[service] !== "healthy") throw new Error(`standing refresh ${service} is not healthy`);

  exactKeys(value.canary, ["url", "status", "required_markers"], "standing refresh.canary");
  requireNonEmptyString(value.canary.url, "standing refresh.canary.url");
  if (value.canary.status !== "pass" || !Array.isArray(value.canary.required_markers) || !["scp-9506", "page-content"].every((marker) => value.canary.required_markers.includes(marker))) {
    throw new Error("standing refresh canary is not passing");
  }

  exactKeys(value.cleanup, ["status", "candidate_receipt", "receipt", "superseded_images"], "standing refresh.cleanup");
  if (value.cleanup.status !== "pass" || !Array.isArray(value.cleanup.superseded_images)) throw new Error("standing refresh cleanup is not passing");
  requirePathDigest(value.cleanup.candidate_receipt, "standing refresh cleanup candidate receipt");
  requirePathDigest(value.cleanup.receipt, "standing refresh cleanup receipt");
  if (JSON.stringify(value.cleanup.candidate_receipt) !== JSON.stringify(value.prepared_receipt) || JSON.stringify(value.cleanup.receipt) !== JSON.stringify(value.prepared_receipt)) {
    throw new Error("standing refresh cleanup identity is not bound to the prepared receipt");
  }

  exactKeys(value.resource_disposition, ["active", "rollback", "volumes", "worktrees", "target_directories"], "standing refresh resource disposition");
  validateResourceImages(value.resource_disposition.active, "standing refresh resource active", value.images);
  validateResourceImages(value.resource_disposition.rollback, "standing refresh resource rollback", value.rollback_images);
  for (const field of ["volumes", "worktrees", "target_directories"]) requireNonEmptyString(value.resource_disposition[field], `standing refresh resource ${field}`);
  validateRefreshRuntimeIdentity(value.runtime_differential_identity, value);
  return value;
}
