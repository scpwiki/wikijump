# Reserved Fixture Backlog Schema

`reserved-fixture-backlog.tsv` must contain these exact columns in this exact order:

| column | required | notes |
| --- | --- | --- |
| slug | yes | Stable fixture identifier (may include `fragment:` prefixes). |
| priority | yes | Integer priority, lower is higher urgency. |
| fixture_type | yes | One of: `simple`, `listpages`, `resource_heavy`, `broad_parity`. |
| source_branch | yes | Source branch used for local corpus derivation (`develop` for this artifact). |
| source_path | yes | Absolute filesystem path to `source.wikidot.txt` in local corpus. |
| local_corpus_status | yes | `present`, `missing`, or `pending-import`. |
| known_dependencies | yes | `;`-separated dependency list; empty allowed as `none`. |
| primary_risk | yes | Concise risk statement for planning. |
| recommended_issue | yes | One of `#4`, `#7`, `#8`, `#6`, `#9`, or `follow-up`. |
| status | yes | Backlog state (`planned`, `in_progress`, `blocked`, `done`). |
| notes | yes | Deterministic free-form notes, no tabs. |

## Validation policy

- Header must match the schema order exactly.
- Slug values must be unique.
- Required slugs from planning: `scp-3352`, `scp-8980`, `scp-anthology-2024`, `scp-9506`.
- If `local_corpus_status == present`, `source_path` must exist.
- `fixture_type`, `local_corpus_status`, and `status` must use known enumerations.
