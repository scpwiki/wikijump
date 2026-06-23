#!/usr/bin/env bash
set -euo pipefail

BACKLOG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TSV="${BACKLOG_DIR}/reserved-fixture-backlog.tsv"
SCHEMA_HEADER=$'slug\tpriority\tfixture_type\tsource_branch\tsource_path\tlocal_corpus_status\tknown_dependencies\tprimary_risk\trecommended_issue\tstatus\tnotes'
if [[ ! -f "${TSV}" ]]; then
  echo "MISSING: ${TSV}" >&2
  exit 1
fi

HEADER=$(head -n 1 "${TSV}")
if [[ "${HEADER}" != "${SCHEMA_HEADER}" ]]; then
  echo "INVALID HEADER" >&2
  echo "Expected: ${SCHEMA_HEADER}" >&2
  echo "Actual:   ${HEADER}" >&2
  exit 1
fi

TSV_PATH="${TSV}" \
python3 - <<'PY'
import csv
import os
import sys

path = os.environ['TSV_PATH']
required = {"scp-3352", "scp-8980", "scp-anthology-2024", "scp-9506"}
allowed_issues = {"#4", "#7", "#8", "#6", "#9", "follow-up"}
allowed_types = {"simple", "listpages", "resource_heavy", "broad_parity"}
allowed_local = {"present", "missing", "pending-import"}
allowed_status = {"planned", "in_progress", "blocked", "done"}

with open(path, newline='', encoding='utf-8') as f:
    raw_rows = list(csv.reader(f, delimiter='\t'))

errors = []
expected_header = [
    'slug',
    'priority',
    'fixture_type',
    'source_branch',
    'source_path',
    'local_corpus_status',
    'known_dependencies',
    'primary_risk',
    'recommended_issue',
    'status',
    'notes',
]
expected_width = len(expected_header)
for line_no, fields in enumerate(raw_rows[1:], start=2):
    if len(fields) != expected_width:
        errors.append(
            f'wrong field count on line {line_no}: expected {expected_width}, got {len(fields)}'
        )

rows = [dict(zip(expected_header, fields)) for fields in raw_rows[1:] if len(fields) == expected_width]

seen = set()
previous_priority = None
for row in rows:
    slug = row['slug'].strip()
    if not slug:
        errors.append('empty slug')
        continue
    if slug in seen:
        errors.append(f'duplicate slug: {slug}')
    seen.add(slug)

    p = row['priority'].strip()
    try:
        priority = int(p)
    except ValueError:
        errors.append(f'non-integer priority for {slug}: {p}')
    else:
        if previous_priority is not None and priority < previous_priority:
            errors.append(f'priority not sorted for {slug}: {priority}')
        previous_priority = priority

    if row['source_branch'].strip() != 'develop':
        errors.append(f'invalid source_branch for {slug}: {row["source_branch"]}')

    if row['fixture_type'] not in allowed_types:
        errors.append(f'invalid fixture_type for {slug}: {row["fixture_type"]}')
    if row['local_corpus_status'] not in allowed_local:
        errors.append(f'invalid local_corpus_status for {slug}: {row["local_corpus_status"]}')
    if row['status'] not in allowed_status:
        errors.append(f'invalid status for {slug}: {row["status"]}')

    if row['recommended_issue'] not in allowed_issues:
        errors.append(f'invalid recommended_issue for {slug}: {row["recommended_issue"]}')

    src = row['source_path'].strip()
    if not src:
        errors.append(f'missing source_path for {slug}')
    elif row['local_corpus_status'] == 'present' and not os.path.isfile(src):
        errors.append(f'missing source file for {slug}: {src}')

for slug in required:
    if not any(r['slug'] == slug for r in rows):
        errors.append(f'required slug missing: {slug}')

if errors:
    print('VALIDATION FAILED')
    for err in errors:
        print(f'- {err}')
    sys.exit(1)

print('VALIDATION PASSED')
print(f'rows={len(rows)}')
print(f'unique_slugs={len(seen)}')
PY
