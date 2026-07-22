#!/usr/bin/env bash

set -euo pipefail

readonly warning_limit=1500
readonly severe_limit=2500
readonly baseline_file="scripts/source-size-baseline.txt"

declare -A severe_baseline=()

while read -r ceiling path; do
    if [[ -z "${ceiling}" || "${ceiling}" == \#* ]]; then
        continue
    fi
    severe_baseline["${path}"]="${ceiling}"
done < "${baseline_file}"

warning_count=0
severe_count=0

while IFS= read -r -d '' path; do
    case "${path}" in
        *.rs|*.ts|*.svelte) ;;
        *) continue ;;
    esac

    case "/${path}/" in
        */test/*|*/tests/*|*/vendor/*|*/generated/*|*/node_modules/*) continue ;;
    esac

    case "${path}" in
        */test.rs|*/tests.rs|*_test.rs|*.test.ts|*.spec.ts|*.generated.ts) continue ;;
    esac

    lines=$(wc -l < "${path}")
    if (( lines <= warning_limit )); then
        continue
    fi

    ((warning_count += 1))
    if (( lines <= severe_limit )); then
        echo "::warning file=${path}::${path} has ${lines} lines; the source-size warning limit is ${warning_limit}."
        continue
    fi

    ceiling=${severe_baseline["${path}"]:-}
    if [[ -n "${ceiling}" ]] && (( lines <= ceiling )); then
        echo "::warning file=${path}::${path} has ${lines} lines; this pre-existing severe violation is grandfathered up to ${ceiling} lines."
        continue
    fi

    ((severe_count += 1))
    if [[ -n "${ceiling}" ]]; then
        echo "::error file=${path}::${path} has grown to ${lines} lines, above its grandfathered ceiling of ${ceiling}."
    else
        echo "::error file=${path}::${path} has ${lines} lines; new severe violations above ${severe_limit} lines are not allowed."
    fi
done < <(git ls-files -z)

echo "Source-size budget: ${warning_count} file(s) above ${warning_limit} lines; ${severe_count} new or grown severe violation(s) above ${severe_limit} lines."

if (( severe_count > 0 )); then
    exit 1
fi
