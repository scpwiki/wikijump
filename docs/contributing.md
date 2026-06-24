# Contributing

This document assumes you have read [Development.md](development.md).

Wikijump has a fairly extensive scope and has lots of areas for people to contribute. Whether it's a small typofix or work on a potential new feature, there's always room for community contributions. If appropriate, a team member can make a [JIRA](https://scuttle.atlassian.net/browse/WJ) issue for it.

It is important that you join the Wikijump Discord so you can discuss and coordinate with the Wikijump team.  You can get an invitation by asking in [#site11](https://scp-wiki.wikidot.com/chat-guide).

Once you've implemented the changes, create a PR and request reviews from 1-3 relevant people. See [CODEOWNERS](../CODEOWNERS) to get an idea of who works on which parts of the repository. It'll be reviewed and merged if ready.

All changes should be merged against `develop`, which automatically deploys to `wikijump.dev`. In longer cycles, we take accrued changes in `develop` and produce a squash commit to `prod`, which deploys to `wikijump.com` (the production environment). This way can utilize continuous deployment for development but also keep production stable. (See [CI.md](ci.md) for more information)

## CodeRabbit review-fix workflow

When a PR receives CodeRabbit review comments, use this repository to keep a small, deterministic fix loop:

- Read the GitHub PR comments and keep a list of actionable findings.
- Apply only fixes required by the review item; avoid broad refactors.
- Keep each fix in a focused commit tied to the review item so the result can be re-run quickly.
- Update any impacted tests only when behavior changed.
- Run the minimal validation command for docs-only or code-only changes, and record the exact command and outcome.

Recommended convention for reviewer-facing follow-up: store a short note in the PR with:

1. review item handled,
2. file(s) changed,
3. validation run,
4. unresolved/invalid findings and rationale.
