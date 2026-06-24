function keyForFinding(finding) {
  return [
    finding.source,
    finding.source_id,
    finding.path ?? "",
    finding.line ?? "",
    finding.body_sha256 ?? "",
  ].join("|");
}

async function sha256Hex(text) {
  const bytes = new TextEncoder().encode(text);
  const hash = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function assertValidTimestamp(value, name) {
  if (typeof value !== "string" || Number.isNaN(Date.parse(value))) {
    throw new Error(`${name} must be a valid ISO-8601 timestamp`);
  }
}

function isAfterMerge(item, mergeAt) {
  const timestamp = item.submitted_at ?? item.created_at ?? item.updated_at;
  return typeof timestamp === "string" && new Date(timestamp) > new Date(mergeAt);
}

function nextHourlyWatchAfter(mergeAt) {
  const date = new Date(mergeAt);
  date.setUTCHours(date.getUTCHours() + 1, 0, 0, 0);
  return date.toISOString();
}

async function normalizeReview(review) {
  return {
    source: "review",
    source_id: String(review.id),
    source_author: review.user?.login ?? null,
    source_state: review.state ?? null,
    type: review.state === "CHANGES_REQUESTED" ? "changes_requested" : "review",
    path: null,
    line: null,
    body: review.body ?? "",
    body_sha256: await sha256Hex(review.body ?? ""),
    created_at: review.submitted_at ?? null,
    controller_disposition: "unverified",
  };
}

async function normalizeComment(comment) {
  return {
    source: "comment",
    source_id: String(comment.id),
    source_author: comment.user?.login ?? null,
    source_state: null,
    type: comment.path ? "inline_comment" : "issue_comment",
    path: comment.path ?? null,
    line: comment.line ?? null,
    body: comment.body ?? "",
    body_sha256: await sha256Hex(comment.body ?? ""),
    created_at: comment.created_at ?? null,
    controller_disposition: "unverified",
  };
}

export async function collectPostMergeFindings({
  pr,
  reviews = [],
  comments = [],
  previousFindingKeys = [],
} = {}) {
  if (typeof pr?.number !== "number") {
    throw new Error("pr.number is required");
  }
  assertValidTimestamp(pr?.merged_at, "pr.merged_at");

  const seen = new Set(previousFindingKeys);
  const findings = [];
  for (const review of reviews.filter((item) => isAfterMerge(item, pr.merged_at))) {
    const finding = await normalizeReview(review);
    const key = keyForFinding(finding);
    if (!seen.has(key)) {
      seen.add(key);
      findings.push({...finding, finding_key: key});
    }
  }
  for (const comment of comments.filter((item) => isAfterMerge(item, pr.merged_at))) {
    const finding = await normalizeComment(comment);
    const key = keyForFinding(finding);
    if (!seen.has(key)) {
      seen.add(key);
      findings.push({...finding, finding_key: key});
    }
  }

  return {
    schema_version: 1,
    pr_number: pr.number,
    merge_commit: pr.merge_commit ?? null,
    head_sha: pr.head_sha ?? null,
    merged_at: pr.merged_at,
    delayed_watch_after: nextHourlyWatchAfter(pr.merged_at),
    findings,
  };
}
