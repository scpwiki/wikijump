const STATES = new Set([
  "ACCEPTED_CLOSED",
  "CLOSED_BUT_UNRECONCILED",
  "OPEN_BUT_ACCEPTED",
  "NEEDS_PROOF",
  "NEEDS_FOLLOWUP_ISSUE",
  "STATE_CONTRADICTION",
]);

function collection(value) {
  return Array.isArray(value) ? value : [];
}

function hasAcceptedProof(proofBundles) {
  return proofBundles.some((proof) => proof?.status === "accepted" || proof?.status === "pass");
}

function hasOpenChild(children) {
  return children.some((child) => String(child?.state ?? "").toUpperCase() === "OPEN");
}

function hasUndisposedGap(gapLedger) {
  return gapLedger.some((gap) => !["fixed", "not_applicable", "accepted_out_of_scope", "successor_issue"].includes(gap?.disposition));
}

function hasUnresolvedPostMergeFinding(postMergeFindings) {
  return postMergeFindings.some((finding) => finding?.disposition === "unresolved" || finding?.needs_followup === true);
}

function ownerCommentContradictsClosure(issue) {
  const comments = collection(issue?.owner_comments);
  return comments.some((comment) => {
    const body = String(comment?.body ?? "").toLowerCase();
    return (
      body.includes("acceptance remained open") ||
      body.includes("acceptance remains open") ||
      body.includes("does not satisfy") ||
      body.includes("did not satisfy") ||
      body.includes("parent acceptance remained open")
    );
  });
}

function isClosed(issue) {
  return String(issue?.state ?? "").toUpperCase() === "CLOSED";
}

function isOpen(issue) {
  return String(issue?.state ?? "").toUpperCase() === "OPEN";
}

function classifyIssue(input = {}) {
  const issue = input?.issue;
  const children = collection(input?.children);
  const proofBundles = collection(input?.proofBundles);
  const gapLedger = collection(input?.gapLedger);
  const postMergeFindings = collection(input?.postMergeFindings);

  if (ownerCommentContradictsClosure(issue)) {
    return "STATE_CONTRADICTION";
  }
  if (hasUnresolvedPostMergeFinding(postMergeFindings)) {
    return "NEEDS_FOLLOWUP_ISSUE";
  }
  if (!hasAcceptedProof(proofBundles) || hasOpenChild(children) || hasUndisposedGap(gapLedger)) {
    if (isClosed(issue)) {
      return "CLOSED_BUT_UNRECONCILED";
    }
    return "NEEDS_PROOF";
  }
  if (isOpen(issue)) {
    return "OPEN_BUT_ACCEPTED";
  }
  if (isClosed(issue)) {
    return "ACCEPTED_CLOSED";
  }
  return "NEEDS_PROOF";
}

export function reconcileIssueClosure(input = {}) {
  const normalizedInput = input ?? {};
  const classification = classifyIssue(normalizedInput);
  if (!STATES.has(classification)) {
    throw new Error(`unsupported classification: ${classification}`);
  }
  return {
    schema_version: 1,
    issue_number: normalizedInput.issue?.number ?? null,
    classification,
    proposed_comment: proposeComment({classification, issue: normalizedInput.issue}),
  };
}

export function proposeComment({classification, issue}) {
  const issueNumber = issue?.number === undefined ? "unknown" : `#${issue.number}`;
  switch (classification) {
    case "ACCEPTED_CLOSED":
      return `${issueNumber} is closed and reconciled against acceptance evidence.`;
    case "CLOSED_BUT_UNRECONCILED":
      return `${issueNumber} is closed, but acceptance evidence or gap disposition is incomplete. Controller review is required before treating it as accepted.`;
    case "OPEN_BUT_ACCEPTED":
      return `${issueNumber} appears accepted by evidence while still open. Controller may prepare a closure comment.`;
    case "NEEDS_PROOF":
      return `${issueNumber} still needs proof or gap disposition before closure.`;
    case "NEEDS_FOLLOWUP_ISSUE":
      return `${issueNumber} has unresolved post-merge findings. Create a narrow follow-up issue before closure.`;
    case "STATE_CONTRADICTION":
      return `${issueNumber} has contradictory closure evidence. Do not reopen or close automatically; controller review is required.`;
    default:
      throw new Error(`unsupported classification: ${classification}`);
  }
}
