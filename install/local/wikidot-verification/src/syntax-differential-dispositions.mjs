export const SYNTAX_VERDICT_SCHEMA = "wikijump_syntax_differential.verdict.v1";
export const DISPOSITION_POLICY_SCHEMA = "wikijump_syntax_differential.disposition_policy.v1";
export const DISPOSITION_VERDICT_SCHEMA = "wikijump_syntax_differential.disposition_verdict.v1";

const STATUSES = new Set(["match", "mismatch", "runner-error", "not-applicable"]);
const DISPOSITIONS = new Set([
  "intentional-security-boundary",
  "wikijump-runtime-boundary",
  "live-observation-resource-failure",
  "live-observation-tier-conflict",
]);

function requireSha256(value, label) {
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/u.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256`);
  }
  return value;
}

export function validateDispositionPolicy(policy) {
  if (policy?.schema !== DISPOSITION_POLICY_SCHEMA || !Array.isArray(policy.cases)) {
    throw new Error("syntax disposition policy is invalid");
  }
  const caseIds = new Set();
  for (const entry of policy.cases) {
    if (
      typeof entry?.case_id !== "string" ||
      !entry.case_id ||
      caseIds.has(entry.case_id) ||
      !DISPOSITIONS.has(entry.disposition) ||
      typeof entry.reason !== "string" ||
      !entry.reason.trim()
    ) {
      throw new Error("syntax disposition policy entry is invalid or duplicated");
    }
    requireSha256(entry.source_sha256, `policy source hash for ${entry.case_id}`);
    caseIds.add(entry.case_id);
  }
  return policy;
}

export function validateSyntaxVerdict(verdict) {
  if (verdict?.schema !== SYNTAX_VERDICT_SCHEMA || !Array.isArray(verdict.comparisons)) {
    throw new Error("syntax differential verdict is invalid");
  }
  const caseIds = new Set();
  for (const comparison of verdict.comparisons) {
    if (
      typeof comparison?.case_id !== "string" ||
      !comparison.case_id ||
      caseIds.has(comparison.case_id) ||
      !STATUSES.has(comparison.status)
    ) {
      throw new Error("syntax differential comparison is invalid or duplicated");
    }
    if (comparison.status !== "not-applicable") {
      requireSha256(
        comparison.identities?.source_sha256,
        `verdict source hash for ${comparison.case_id}`,
      );
    }
    caseIds.add(comparison.case_id);
  }
  return verdict;
}

export function classifySyntaxDispositions(verdictInput, policyInput) {
  const verdict = validateSyntaxVerdict(verdictInput);
  const policy = validateDispositionPolicy(policyInput);
  const policyByCaseId = new Map(policy.cases.map((entry) => [entry.case_id, entry]));
  const seenPolicyCaseIds = new Set();
  const accepted = [];
  const failures = [];

  for (const comparison of verdict.comparisons) {
    const entry = policyByCaseId.get(comparison.case_id);
    if (entry) seenPolicyCaseIds.add(entry.case_id);

    if (comparison.status === "mismatch") {
      if (!entry) {
        failures.push({case_id: comparison.case_id, kind: "unknown-mismatch"});
      } else if (entry.source_sha256 !== comparison.identities.source_sha256) {
        failures.push({
          case_id: comparison.case_id,
          kind: "stale-source-hash",
          policy_source_sha256: entry.source_sha256,
          verdict_source_sha256: comparison.identities.source_sha256,
        });
      } else {
        accepted.push({
          case_id: comparison.case_id,
          source_sha256: entry.source_sha256,
          disposition: entry.disposition,
          reason: entry.reason,
        });
      }
    } else if (entry) {
      failures.push({
        case_id: comparison.case_id,
        kind: comparison.status === "match" ? "resolved-policy-entry" : "inapplicable-policy-entry",
        status: comparison.status,
      });
    }

    if (comparison.status === "runner-error") {
      failures.push({case_id: comparison.case_id, kind: "runner-error"});
    }
  }

  for (const entry of policy.cases) {
    if (!seenPolicyCaseIds.has(entry.case_id)) {
      failures.push({case_id: entry.case_id, kind: "missing-policy-case"});
    }
  }

  return {
    schema: DISPOSITION_VERDICT_SCHEMA,
    status: failures.length === 0 ? "accepted" : "failed",
    accepted,
    failures,
    summary: {
      comparisons: verdict.comparisons.length,
      accepted_mismatches: accepted.length,
      failures: failures.length,
    },
  };
}
